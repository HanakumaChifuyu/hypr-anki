use axum::extract::{
    OriginalUri, Path, State,
    rejection::{JsonRejection, PathRejection},
};
use axum::http::StatusCode;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use hypranki_service::services::deck::DeckNotFound;
use hypranki_storage::models::deck::{CreateDeck, Deck, UpdateDeck};
use serde::Deserialize;
use tracing::{info, warn};

use super::app_error::AppError;
use crate::service::ServiceState;

#[derive(Deserialize, utoipa::ToSchema)]
struct RenameDeck {
    name: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct MoveDeck {
    // 显式 null 表示移动到根目录；不传字段视为参数错误。
    #[serde(deserialize_with = "Option::<i64>::deserialize")]
    #[schema(required = true, nullable = true)]
    parent_id: Option<i64>,
}

#[derive(utoipa::ToSchema)]
struct ErrorResponse {
    /// 请求失败时为 false。
    success: bool,
    message: String,
}

#[derive(utoipa::OpenApi)]
#[openapi(
    info(title = "HyprAnki API", description = "牌组管理接口"),
    paths(list_deck, get_deck, create_deck,
        update_deck, rename_deck, move_deck,
        delete_subtree),
    tags((name = "牌组", description = "牌组与文件夹的查询和管理"))
)]
pub struct ApiDoc;

pub struct DeckHandler;

impl DeckHandler {
    pub fn router() -> Router<ServiceState> {
        Router::new()
            .route("/list", get(list_deck))
            .route("/", post(create_deck))
            .route(
                "/{id}",
                get(get_deck).patch(update_deck).delete(delete_subtree),
            )
            .route("/{id}/rename", patch(rename_deck))
            .route("/{id}/move", patch(move_deck))
    }

    // 数据库错误优先处理，避免泄露内部信息；目标不存在有独立的类型。
    fn service_error(error: anyhow::Error) -> AppError {
        if error.downcast_ref::<sqlx::Error>().is_some() {
            AppError::InternalError(error)
        } else if error.downcast_ref::<DeckNotFound>().is_some() {
            warn!(status = 404, error = %error, "牌组不存在");
            AppError::NotFound(error.to_string())
        } else {
            warn!(status = 400, error = %error, "牌组操作校验失败");
            AppError::BadRequest(error.to_string())
        }
    }

    fn id(path: Result<Path<i64>, PathRejection>) -> Result<i64, AppError> {
        let Path(id) = path.map_err(|error| AppError::BadRequest(error.body_text()))?;
        if id <= 0 {
            return Err(AppError::BadRequest("牌组 ID 必须为正整数".into()));
        }
        Ok(id)
    }

    fn body<T>(body: Result<Json<T>, JsonRejection>) -> Result<T, AppError> {
        body.map(|Json(value)| value)
            .map_err(|error| AppError::BadRequest(error.body_text()))
    }
}

#[utoipa::path(get, path = "/deck/list", tag = "牌组", summary = "查询牌组列表（先序排列）",
    responses((status = 200, description = "成功", body = [Deck]),
        (status = 500, description = "内部服务错误", body = ErrorResponse))) ]
#[tracing::instrument(name = "查询牌组列表", skip_all, fields(method = "GET", path = %uri.path()))]
async fn list_deck(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
) -> Result<Json<Vec<Deck>>, AppError> {
    info!("开始查询牌组列表");
    // 无用户输入的查询失败意味着数据库或已有树数据异常。
    let decks = state
        .deck_service
        .list()
        .await
        .map_err(AppError::InternalError)?;
    info!(status = 200, count = decks.len(), "查询牌组列表成功");
    Ok(Json(decks))
}

#[utoipa::path(get, path = "/deck/{id}", tag = "牌组", summary = "获取单个牌组",
    params(("id" = i64, Path, description = "牌组 ID（正整数）", minimum = 1)),
    responses((status = 200, description = "成功", body = Deck),
        (status = 400, description = "参数或业务校验失败", body = ErrorResponse),
        (status = 404, description = "目标牌组不存在", body = ErrorResponse),
        (status = 500, description = "内部服务错误", body = ErrorResponse))) ]
#[tracing::instrument(name = "查询单个牌组", skip_all, fields(method = "GET", path = %uri.path()))]
async fn get_deck(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
) -> Result<Json<Deck>, AppError> {
    let id = DeckHandler::id(path)?;
    let deck = state
        .deck_service
        .get(id)
        .await
        .map_err(DeckHandler::service_error)?;
    info!(status = 200, id, "查询牌组成功");
    Ok(Json(deck))
}

#[utoipa::path(post, path = "/deck", tag = "牌组", summary = "创建牌组",
    request_body = CreateDeck,
    responses((status = 201, description = "成功", body = Deck),
        (status = 400, description = "参数或业务校验失败", body = ErrorResponse),
        (status = 500, description = "内部服务错误", body = ErrorResponse))) ]
#[tracing::instrument(name = "创建牌组", skip_all, fields(method = "POST", path = %uri.path()))]
async fn create_deck(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    body: Result<Json<CreateDeck>, JsonRejection>,
) -> Result<(StatusCode, Json<Deck>), AppError> {
    let input = DeckHandler::body(body)?;
    let deck = state
        .deck_service
        .create(&input)
        .await
        .map_err(DeckHandler::service_error)?;
    info!(status = 201, id = deck.id, "创建牌组成功");
    Ok((StatusCode::CREATED, Json(deck)))
}

#[utoipa::path(patch, path = "/deck/{id}", tag = "牌组", summary = "部分更新牌组；省略 parent_id 保持不变，null 移到根目录",
    params(("id" = i64, Path, description = "牌组 ID（正整数）", minimum = 1)),
    request_body = UpdateDeck,
    responses((status = 200, description = "成功", body = Deck),
        (status = 400, description = "参数或业务校验失败", body = ErrorResponse),
        (status = 404, description = "目标牌组不存在", body = ErrorResponse),
        (status = 500, description = "内部服务错误", body = ErrorResponse))) ]
#[tracing::instrument(name = "更新牌组", skip_all, fields(method = "PATCH", path = %uri.path()))]
async fn update_deck(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
    body: Result<Json<UpdateDeck>, JsonRejection>,
) -> Result<Json<Deck>, AppError> {
    let id = DeckHandler::id(path)?;
    let input = DeckHandler::body(body)?;
    let deck = state
        .deck_service
        .update(id, &input)
        .await
        .map_err(DeckHandler::service_error)?;
    info!(status = 200, id, "更新牌组成功");
    Ok(Json(deck))
}

#[utoipa::path(patch, path = "/deck/{id}/rename", tag = "牌组", summary = "重命名牌组",
    params(("id" = i64, Path, description = "牌组 ID（正整数）", minimum = 1)),
    request_body = RenameDeck,
    responses((status = 200, description = "成功", body = Deck),
        (status = 400, description = "参数或业务校验失败", body = ErrorResponse),
        (status = 404, description = "目标牌组不存在", body = ErrorResponse),
        (status = 500, description = "内部服务错误", body = ErrorResponse))) ]
#[tracing::instrument(name = "重命名牌组", skip_all, fields(method = "PATCH", path = %uri.path()))]
async fn rename_deck(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
    body: Result<Json<RenameDeck>, JsonRejection>,
) -> Result<Json<Deck>, AppError> {
    let id = DeckHandler::id(path)?;
    let input = DeckHandler::body(body)?;
    let deck = state
        .deck_service
        .rename(id, input.name)
        .await
        .map_err(DeckHandler::service_error)?;
    info!(status = 200, id, "重命名牌组成功");
    Ok(Json(deck))
}

#[utoipa::path(patch, path = "/deck/{id}/move", tag = "牌组", summary = "移动牌组及子树；parent_id 必传，null 表示根目录",
    params(("id" = i64, Path, description = "牌组 ID（正整数）", minimum = 1)),
    request_body = MoveDeck,
    responses((status = 200, description = "成功", body = Deck),
        (status = 400, description = "参数或业务校验失败", body = ErrorResponse),
        (status = 404, description = "目标牌组不存在", body = ErrorResponse),
        (status = 500, description = "内部服务错误", body = ErrorResponse))) ]
#[tracing::instrument(name = "移动牌组子树", skip_all, fields(method = "PATCH", path = %uri.path()))]
async fn move_deck(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
    body: Result<Json<MoveDeck>, JsonRejection>,
) -> Result<Json<Deck>, AppError> {
    let id = DeckHandler::id(path)?;
    let input = DeckHandler::body(body)?;
    let parent_id = input.parent_id;
    let deck = state
        .deck_service
        .move_to(id, parent_id)
        .await
        .map_err(DeckHandler::service_error)?;
    info!(status = 200, id, ?parent_id, "移动牌组成功");
    Ok(Json(deck))
}

#[utoipa::path(delete, path = "/deck/{id}", tag = "牌组", summary = "删除牌组及全部后代，返回删除的 ID",
    params(("id" = i64, Path, description = "牌组 ID（正整数）", minimum = 1)),
    responses((status = 200, description = "成功", body = [i64]),
        (status = 400, description = "参数或业务校验失败", body = ErrorResponse),
        (status = 404, description = "目标牌组不存在", body = ErrorResponse),
        (status = 500, description = "内部服务错误", body = ErrorResponse))) ]
#[tracing::instrument(name = "删除牌组子树", skip_all, fields(method = "DELETE", path = %uri.path()))]
async fn delete_subtree(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
) -> Result<Json<Vec<i64>>, AppError> {
    let id = DeckHandler::id(path)?;
    let deleted = state
        .deck_service
        .delete_subtree(id)
        .await
        .map_err(DeckHandler::service_error)?;
    info!(status = 200, id, count = deleted.len(), "删除牌组子树成功");
    Ok(Json(deleted))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use hypranki_service::services::deck::DeckService;
    use hypranki_storage::{db::pool::SqlitePool, repository::deck::DeckRepository};
    use serde_json::{Value, json};
    use std::sync::Arc;
    use tower::ServiceExt;

    async fn setup() -> (Router, SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let pool = SqlitePool::new(dir.path().join("test.db")).await.unwrap();
        pool.initialize_database().await.unwrap();
        let state = ServiceState::from_pool(pool.clone());
        let router = Router::new()
            .nest("/deck", DeckHandler::router())
            .with_state(state);
        (router, pool, dir)
    }

    async fn request(
        router: &Router,
        method: &str,
        path: &str,
        body: Value,
    ) -> (StatusCode, Value) {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn routes_cover_deck_lifecycle() {
        let (router, _pool, _dir) = setup().await;
        assert_eq!(
            request(&router, "GET", "/deck/list", Value::Null).await,
            (StatusCode::OK, json!([]))
        );
        let (status, root) = request(
            &router,
            "POST",
            "/deck",
            json!({"name":"Root", "folder":true, "config_id":1}),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        let root_id = root["id"].as_i64().unwrap();
        let (status, child) = request(
            &router,
            "POST",
            "/deck",
            json!({"name":"Child", "config_id":1, "parent_id":root_id}),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        let id = child["id"].as_i64().unwrap();
        let path = format!("/deck/{id}");
        assert_eq!(request(&router, "GET", &path, Value::Null).await.1, child);
        assert_eq!(
            request(
                &router,
                "PATCH",
                &format!("{path}/rename"),
                json!({"name":"Renamed"})
            )
            .await
            .1["name"],
            "Renamed"
        );
        let (status, updated) = request(&router, "PATCH", &path, json!({"config_id":2})).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(updated["parent_id"], root_id);
        let (status, moved) = request(&router, "PATCH", &path, json!({"parent_id":null})).await;
        assert_eq!(status, StatusCode::OK);
        assert!(moved["parent_id"].is_null());
        assert_eq!(
            request(
                &router,
                "PATCH",
                &format!("{path}/move"),
                json!({"parent_id":root_id})
            )
            .await
            .0,
            StatusCode::OK
        );
        assert_eq!(
            request(&router, "DELETE", &format!("/deck/{root_id}"), Value::Null).await,
            (StatusCode::OK, json!([root_id, id]))
        );
        assert_eq!(
            request(&router, "GET", &path, Value::Null).await.0,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn routes_return_structured_errors() {
        let (router, pool, _dir) = setup().await;
        for (method, path, body, expected) in [
            ("GET", "/deck/abc", json!(null), StatusCode::BAD_REQUEST),
            ("GET", "/deck/0", json!(null), StatusCode::BAD_REQUEST),
            ("GET", "/deck/999", json!(null), StatusCode::NOT_FOUND),
            ("PATCH", "/deck/999", json!({}), StatusCode::NOT_FOUND),
            ("DELETE", "/deck/999", json!(null), StatusCode::NOT_FOUND),
            (
                "POST",
                "/deck",
                json!({"name":" ","config_id":1}),
                StatusCode::BAD_REQUEST,
            ),
            (
                "POST",
                "/deck",
                json!({"name":"X","config_id":1,"parent_id":999}),
                StatusCode::BAD_REQUEST,
            ),
            ("POST", "/deck", json!({}), StatusCode::BAD_REQUEST),
            ("PATCH", "/deck/1/move", json!({}), StatusCode::BAD_REQUEST),
            (
                "PATCH",
                "/deck/1/move",
                json!({"parent_id":"bad"}),
                StatusCode::BAD_REQUEST,
            ),
        ] {
            let (status, body) = request(&router, method, path, body).await;
            assert_eq!(status, expected, "{method} {path}: {body}");
            assert_eq!(body["success"], false);
        }
        pool.inner().close().await;
        for path in ["/deck/list", "/deck/1"] {
            let (status, body) = request(&router, "GET", path, Value::Null).await;
            assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
            assert_eq!(body, json!({"success":false,"message":"内部服务错误"}));
        }
    }
}
