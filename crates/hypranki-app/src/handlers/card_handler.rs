use super::app_error::AppError;
use crate::service::ServiceState;
use axum::{
    Json, Router,
    extract::{
        OriginalUri, Path, Query, State,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::StatusCode,
    routing::{get, patch, post},
};
use hypranki_service::services::card::{
    CardDetails, CardError, CardListQuery, CardPage, CreateCardInput, ReviewAnswer, ReviewResult,
};
use hypranki_storage::{models::card::Card, repository::card::CardReview};
use serde::Deserialize;
use serde_json::Value;
use tracing::{info, warn};

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
struct UpdateContent {
    /// 完整字段对象，包含未改变的只读字段。
    content: Value,
    #[serde(default)]
    reset_memory: bool,
}
#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
struct MoveCard {
    deck_id: i64,
}
#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
struct ChangeTemplate {
    template_id: i64,
}
#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
struct DuplicateCard {
    deck_id: i64,
    template_id: i64,
}
#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
struct ReviewCard {
    answer: ReviewAnswer,
}
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct DueQuery {
    limit: u32,
}
impl Default for DueQuery {
    fn default() -> Self {
        Self { limit: 50 }
    }
}
#[derive(utoipa::ToSchema)]
struct CardErrorResponse {
    success: bool,
    message: String,
}

pub struct CardHandler;
impl CardHandler {
    pub fn router() -> Router<ServiceState> {
        Router::new()
            .route("/", post(create_card))
            .route("/list", get(list_cards))
            .route("/due", get(list_due))
            .route("/{id}", get(get_card).delete(delete_card))
            .route("/{id}/content", patch(update_content))
            .route("/{id}/move", patch(move_card))
            .route("/{id}/template", patch(change_template))
            .route("/{id}/duplicate", post(duplicate_card))
            .route("/{id}/reset-memory", post(reset_memory))
            .route("/{id}/review", post(review_card))
            .route("/{id}/review/preview", post(preview_review))
            .route("/{id}/reviews", get(review_history))
    }
    fn service_error(error: anyhow::Error) -> AppError {
        match error.downcast_ref::<CardError>() {
            Some(CardError::NotFound(_)) => {
                warn!(status = 404, error = %error, "卡片不存在");
                AppError::NotFound("卡片不存在或已被删除".into())
            }
            Some(CardError::InvalidInput(message)) => {
                warn!(status = 400, error = ?error, "卡片操作校验失败");
                AppError::BadRequest(message.clone())
            }
            // 数据库、FSRS、损坏的记忆状态和其他意外错误均不向客户端暴露细节。
            _ => AppError::InternalError(error),
        }
    }
    fn positive_id(id: i64) -> Result<i64, AppError> {
        if id <= 0 {
            return Err(AppError::BadRequest("请选择有效的卡片、牌组或模板".into()));
        }
        Ok(id)
    }
    fn id(path: Result<Path<i64>, PathRejection>) -> Result<i64, AppError> {
        let Path(id) = path.map_err(|error| {
            warn!(error = %error, "卡片路径参数无效");
            AppError::BadRequest("无法识别要操作的卡片".into())
        })?;
        Self::positive_id(id)
    }
    fn body<T>(body: Result<Json<T>, JsonRejection>) -> Result<T, AppError> {
        body.map(|Json(value)| value).map_err(|error| {
            warn!(error = %error, "卡片请求数据格式错误");
            AppError::BadRequest("请求数据格式不正确，请检查必填项和字段类型".into())
        })
    }
    fn query<T>(query: Result<Query<T>, QueryRejection>) -> Result<T, AppError> {
        query.map(|Query(value)| value).map_err(|error| {
            warn!(error = %error, "卡片查询条件格式错误");
            AppError::BadRequest("查询条件格式不正确，请检查分页和筛选条件".into())
        })
    }
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(create_card, list_cards, list_due, get_card, update_content, move_card, change_template, duplicate_card, delete_card, reset_memory, review_card, preview_review, review_history), tags((name = "卡片", description = "卡片管理与后端 FSRS 复习")))]
pub struct ApiDoc;
#[utoipa::path(post, path = "/card", tag = "卡片", summary = "创建卡片",
request_body = CreateCardInput,
responses((status = 201, description = "成功", body = Card), (status = 400, description = "参数或业务校验失败", body = CardErrorResponse),(status = 500, description = "内部服务错误", body = CardErrorResponse)))]
#[tracing::instrument(name = "创建卡片", skip_all, fields(method = "POST", path = %uri.path()))]
async fn create_card(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    body: Result<Json<CreateCardInput>, JsonRejection>,
) -> Result<(StatusCode, Json<Card>), AppError> {
    let input = CardHandler::body(body)?;
    CardHandler::positive_id(input.note_type_id)?;
    CardHandler::positive_id(input.template_id)?;
    CardHandler::positive_id(input.deck_id)?;
    let result = state
        .card_service
        .create(&input)
        .await
        .map_err(CardHandler::service_error)?;
    info!(status = 201, "创建卡片成功");
    Ok((StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/card/list", tag = "卡片", summary = "分页查询卡片",
params(("deck_id" = Option<i64>, Query, description = "牌组筛选", minimum = 1), ("page" = Option<u32>, Query, description = "页码，默认 1", minimum = 1), ("page_size" = Option<u32>, Query, description = "每页数量，默认 50", minimum = 1, maximum = 200)),
responses((status = 200, description = "成功", body = CardPage), (status = 400, description = "参数或业务校验失败", body = CardErrorResponse),(status = 500, description = "内部服务错误", body = CardErrorResponse)))]
#[tracing::instrument(name = "分页查询卡片", skip_all, fields(method = "GET", path = %uri.path()))]
async fn list_cards(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    query: Result<Query<CardListQuery>, QueryRejection>,
) -> Result<Json<CardPage>, AppError> {
    let query = CardHandler::query(query)?;
    let result = state
        .card_service
        .list(&query)
        .await
        .map_err(CardHandler::service_error)?;
    info!(status = 200, "分页查询卡片成功");
    Ok(Json(result))
}

#[utoipa::path(get, path = "/card/due", tag = "卡片", summary = "查询到期卡片",
params(("limit" = Option<u32>, Query, description = "返回数量，默认 50", minimum = 1, maximum = 200)),
responses((status = 200, description = "成功", body = [Card]), (status = 400, description = "参数或业务校验失败", body = CardErrorResponse),(status = 500, description = "内部服务错误", body = CardErrorResponse)))]
#[tracing::instrument(name = "查询到期卡片", skip_all, fields(method = "GET", path = %uri.path()))]
async fn list_due(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    query: Result<Query<DueQuery>, QueryRejection>,
) -> Result<Json<Vec<Card>>, AppError> {
    let query = CardHandler::query(query)?;
    if !(1..=200).contains(&query.limit) {
        return Err(AppError::BadRequest("limit 必须在 1 到 200 之间".into()));
    }
    let result = state
        .card_service
        .list_due(query.limit)
        .await
        .map_err(CardHandler::service_error)?;
    info!(status = 200, "查询到期卡片成功");
    Ok(Json(result))
}

#[utoipa::path(get, path = "/card/{id}", tag = "卡片", summary = "查询卡片详情",
params(("id" = i64, Path, description = "卡片 ID", minimum = 1)),
responses((status = 200, description = "成功", body = CardDetails), (status = 400, description = "参数或业务校验失败", body = CardErrorResponse),(status = 404, description = "卡片不存在", body = CardErrorResponse),(status = 500, description = "内部服务错误", body = CardErrorResponse)))]
#[tracing::instrument(name = "查询卡片详情", skip_all, fields(method = "GET", path = %uri.path()))]
async fn get_card(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
) -> Result<Json<CardDetails>, AppError> {
    let id = CardHandler::id(path)?;
    let result = state
        .card_service
        .get(id)
        .await
        .map_err(CardHandler::service_error)?;
    info!(status = 200, "查询卡片详情成功");
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/card/{id}/content", tag = "卡片", summary = "更新卡片完整内容",
params(("id" = i64, Path, description = "卡片 ID", minimum = 1)),
request_body = UpdateContent,
responses((status = 200, description = "成功", body = Card), (status = 400, description = "参数或业务校验失败", body = CardErrorResponse),(status = 404, description = "卡片不存在", body = CardErrorResponse),(status = 500, description = "内部服务错误", body = CardErrorResponse)))]
#[tracing::instrument(name = "更新卡片完整内容", skip_all, fields(method = "PATCH", path = %uri.path()))]
async fn update_content(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
    body: Result<Json<UpdateContent>, JsonRejection>,
) -> Result<Json<Card>, AppError> {
    let id = CardHandler::id(path)?;
    let input = CardHandler::body(body)?;
    let result = state
        .card_service
        .update_content(id, input.content, input.reset_memory)
        .await
        .map_err(CardHandler::service_error)?;
    info!(status = 200, "更新卡片完整内容成功");
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/card/{id}/move", tag = "卡片", summary = "移动卡片",
params(("id" = i64, Path, description = "卡片 ID", minimum = 1)),
request_body = MoveCard,
responses((status = 200, description = "成功", body = Card), (status = 400, description = "参数或业务校验失败", body = CardErrorResponse),(status = 404, description = "卡片不存在", body = CardErrorResponse),(status = 500, description = "内部服务错误", body = CardErrorResponse)))]
#[tracing::instrument(name = "移动卡片", skip_all, fields(method = "PATCH", path = %uri.path()))]
async fn move_card(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
    body: Result<Json<MoveCard>, JsonRejection>,
) -> Result<Json<Card>, AppError> {
    let id = CardHandler::id(path)?;
    let input = CardHandler::body(body)?;
    CardHandler::positive_id(input.deck_id)?;
    let result = state
        .card_service
        .move_to(id, input.deck_id)
        .await
        .map_err(CardHandler::service_error)?;
    info!(status = 200, "移动卡片成功");
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/card/{id}/template", tag = "卡片", summary = "更换卡片模板",
params(("id" = i64, Path, description = "卡片 ID", minimum = 1)),
request_body = ChangeTemplate,
responses((status = 200, description = "成功", body = Card), (status = 400, description = "参数或业务校验失败", body = CardErrorResponse),(status = 404, description = "卡片不存在", body = CardErrorResponse),(status = 500, description = "内部服务错误", body = CardErrorResponse)))]
#[tracing::instrument(name = "更换卡片模板", skip_all, fields(method = "PATCH", path = %uri.path()))]
async fn change_template(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
    body: Result<Json<ChangeTemplate>, JsonRejection>,
) -> Result<Json<Card>, AppError> {
    let id = CardHandler::id(path)?;
    let input = CardHandler::body(body)?;
    CardHandler::positive_id(input.template_id)?;
    let result = state
        .card_service
        .change_template(id, input.template_id)
        .await
        .map_err(CardHandler::service_error)?;
    info!(status = 200, "更换卡片模板成功");
    Ok(Json(result))
}

#[utoipa::path(post, path = "/card/{id}/duplicate", tag = "卡片", summary = "复制为独立卡片",
params(("id" = i64, Path, description = "卡片 ID", minimum = 1)),
request_body = DuplicateCard,
responses((status = 201, description = "成功", body = Card), (status = 400, description = "参数或业务校验失败", body = CardErrorResponse),(status = 404, description = "卡片不存在", body = CardErrorResponse),(status = 500, description = "内部服务错误", body = CardErrorResponse)))]
#[tracing::instrument(name = "复制为独立卡片", skip_all, fields(method = "POST", path = %uri.path()))]
async fn duplicate_card(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
    body: Result<Json<DuplicateCard>, JsonRejection>,
) -> Result<(StatusCode, Json<Card>), AppError> {
    let id = CardHandler::id(path)?;
    let input = CardHandler::body(body)?;
    CardHandler::positive_id(input.deck_id)?;
    CardHandler::positive_id(input.template_id)?;
    let result = state
        .card_service
        .duplicate(id, input.deck_id, input.template_id)
        .await
        .map_err(CardHandler::service_error)?;
    info!(status = 201, "复制为独立卡片成功");
    Ok((StatusCode::CREATED, Json(result)))
}

#[utoipa::path(delete, path = "/card/{id}", tag = "卡片", summary = "删除卡片及独占内容",
params(("id" = i64, Path, description = "卡片 ID", minimum = 1)),
responses((status = 204, description = "成功"), (status = 400, description = "参数或业务校验失败", body = CardErrorResponse),(status = 404, description = "卡片不存在", body = CardErrorResponse),(status = 500, description = "内部服务错误", body = CardErrorResponse)))]
#[tracing::instrument(name = "删除卡片及独占内容", skip_all, fields(method = "DELETE", path = %uri.path()))]
async fn delete_card(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, AppError> {
    let id = CardHandler::id(path)?;
    state
        .card_service
        .delete(id)
        .await
        .map_err(CardHandler::service_error)?;
    info!(status = 204, "删除卡片及独占内容成功");
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/card/{id}/reset-memory", tag = "卡片", summary = "重置卡片记忆并保留历史",
params(("id" = i64, Path, description = "卡片 ID", minimum = 1)),
responses((status = 200, description = "成功", body = Card), (status = 400, description = "参数或业务校验失败", body = CardErrorResponse),(status = 404, description = "卡片不存在", body = CardErrorResponse),(status = 500, description = "内部服务错误", body = CardErrorResponse)))]
#[tracing::instrument(name = "重置卡片记忆并保留历史", skip_all, fields(method = "POST", path = %uri.path()))]
async fn reset_memory(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
) -> Result<Json<Card>, AppError> {
    let id = CardHandler::id(path)?;
    let result = state
        .card_service
        .reset_memory(id)
        .await
        .map_err(CardHandler::service_error)?;
    info!(status = 200, "重置卡片记忆并保留历史成功");
    Ok(Json(result))
}

#[utoipa::path(post, path = "/card/{id}/review", tag = "卡片", summary = "提交复习并更新记忆状态",
params(("id" = i64, Path, description = "卡片 ID", minimum = 1)),
request_body = ReviewCard,
responses((status = 200, description = "成功", body = ReviewResult), (status = 400, description = "参数或业务校验失败", body = CardErrorResponse),(status = 404, description = "卡片不存在", body = CardErrorResponse),(status = 500, description = "内部服务错误", body = CardErrorResponse)))]
#[tracing::instrument(name = "提交复习并更新记忆状态", skip_all, fields(method = "POST", path = %uri.path()))]
async fn review_card(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
    body: Result<Json<ReviewCard>, JsonRejection>,
) -> Result<Json<ReviewResult>, AppError> {
    let id = CardHandler::id(path)?;
    let input = CardHandler::body(body)?;
    let result = state
        .card_service
        .review(id, input.answer)
        .await
        .map_err(CardHandler::service_error)?;
    info!(status = 200, "提交复习并更新记忆状态成功");
    Ok(Json(result))
}

#[utoipa::path(post, path = "/card/{id}/review/preview", tag = "卡片", summary = "预览复习结果（不写入）",
params(("id" = i64, Path, description = "卡片 ID", minimum = 1)),
request_body = ReviewCard,
responses((status = 200, description = "成功", body = ReviewResult), (status = 400, description = "参数或业务校验失败", body = CardErrorResponse),(status = 404, description = "卡片不存在", body = CardErrorResponse),(status = 500, description = "内部服务错误", body = CardErrorResponse)))]
#[tracing::instrument(name = "预览复习结果（不写入）", skip_all, fields(method = "POST", path = %uri.path()))]
async fn preview_review(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
    body: Result<Json<ReviewCard>, JsonRejection>,
) -> Result<Json<ReviewResult>, AppError> {
    let id = CardHandler::id(path)?;
    let input = CardHandler::body(body)?;
    let result = state
        .card_service
        .preview_review(id, input.answer)
        .await
        .map_err(CardHandler::service_error)?;
    info!(status = 200, "预览复习结果（不写入）成功");
    Ok(Json(result))
}

#[utoipa::path(get, path = "/card/{id}/reviews", tag = "卡片", summary = "查询卡片复习历史",
params(("id" = i64, Path, description = "卡片 ID", minimum = 1)),
responses((status = 200, description = "成功", body = [CardReview]), (status = 400, description = "参数或业务校验失败", body = CardErrorResponse),(status = 404, description = "卡片不存在", body = CardErrorResponse),(status = 500, description = "内部服务错误", body = CardErrorResponse)))]
#[tracing::instrument(name = "查询卡片复习历史", skip_all, fields(method = "GET", path = %uri.path()))]
async fn review_history(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
) -> Result<Json<Vec<CardReview>>, AppError> {
    let id = CardHandler::id(path)?;
    let result = state
        .card_service
        .review_history(id)
        .await
        .map_err(CardHandler::service_error)?;
    info!(status = 200, "查询卡片复习历史成功");
    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use hypranki_storage::db::pool::SqlitePool;
    use serde_json::json;
    use tower::ServiceExt;

    async fn setup() -> (Router, SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let pool = SqlitePool::new(dir.path().join("handlers.db"))
            .await
            .unwrap();
        pool.initialize_database().await.unwrap();
        sqlx::raw_sql(r#"
            INSERT INTO DECKS (id, name, folder, config_id, created_at, updated_at) VALUES (1,'A',0,1,0,0),(2,'B',0,1,0,0),(3,'Folder',1,1,0,0);
            INSERT INTO NOTETYPES (id, name, fields) VALUES (1,'Basic','[{"name":"Front","field_type":"Str","writability":true},{"name":"Source","field_type":"Str","writability":false}]');
            INSERT INTO TEMPLATES (id,name) VALUES (1,'A'),(2,'B');
        "#).execute(pool.inner()).await.unwrap();
        let router = crate::handlers::AppHandler::router(ServiceState::from_pool(pool.clone()));
        (router, pool, dir)
    }
    async fn request(router: &Router, method: &str, path: &str, body: &str) -> (StatusCode, Value) {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_owned()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        (
            status,
            if bytes.is_empty() {
                Value::Null
            } else {
                serde_json::from_slice(&bytes).unwrap()
            },
        )
    }
    fn input() -> String {
        json!({"note_type_id":1,"template_id":1,"deck_id":1,"content":{"Front":"hello","Source":"book"}}).to_string()
    }

    #[tokio::test]
    async fn card_routes_lifecycle_and_pagination() {
        let (router, _pool, _dir) = setup().await;
        let (status, card) = request(&router, "POST", "/card", &input()).await;
        assert_eq!(status, StatusCode::CREATED);
        let id = card["id"].as_i64().unwrap();
        let path = format!("/card/{id}");
        let (status, details) = request(&router, "GET", &path, "").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(details["content"]["Front"], "hello");
        assert_eq!(details["deck_name"], "A");
        assert!(details.get("note_id").is_none());
        let (status, page) = request(
            &router,
            "GET",
            "/card/list?deck_id=1&page=1&page_size=1",
            "",
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(page["total"], 1);
        assert_eq!(page["items"][0]["id"], id);
        let (_, empty) = request(&router, "GET", "/card/list?page=2&page_size=1", "").await;
        assert_eq!(empty["items"], json!([]));
        assert_eq!(empty["total"], 1);
        let (status, updated) = request(
            &router,
            "PATCH",
            &format!("{path}/content"),
            r#"{"content":{"Front":"changed","Source":"book"}}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(updated["id"], id);
        let (status, moved) = request(
            &router,
            "PATCH",
            &format!("{path}/move"),
            r#"{"deck_id":2}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(moved["deck_id"], 2);
        let (status, template) = request(
            &router,
            "PATCH",
            &format!("{path}/template"),
            r#"{"template_id":2}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(template["template_id"], 2);
        let (status, copy) = request(
            &router,
            "POST",
            &format!("{path}/duplicate"),
            r#"{"deck_id":1,"template_id":1}"#,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_ne!(copy["note_id"], card["note_id"]);
        assert!(copy["memory_state"].is_null());
        assert_eq!(
            request(&router, "DELETE", &path, "").await,
            (StatusCode::NO_CONTENT, Value::Null)
        );
        assert_eq!(
            request(&router, "GET", &path, "").await.0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            request(&router, "GET", &format!("/card/{}", copy["id"]), "")
                .await
                .1["content"]["Front"],
            "changed"
        );
    }

    #[tokio::test]
    async fn reviews_preview_history_due_and_reset() {
        let (router, pool, _dir) = setup().await;
        let (_, card) = request(&router, "POST", "/card", &input()).await;
        let id = card["id"].as_i64().unwrap();
        let path = format!("/card/{id}");
        let answer = r#"{"answer":"remembered"}"#;
        let (status, preview) =
            request(&router, "POST", &format!("{path}/review/preview"), answer).await;
        assert_eq!(status, StatusCode::OK);
        assert!(preview["due_at"].as_i64().unwrap() > 0);
        assert_eq!(
            request(&router, "GET", &format!("{path}/reviews"), "").await,
            (StatusCode::OK, json!([]))
        );
        assert!(request(&router, "GET", &path, "").await.1["last_reviewed_at"].is_null());
        let (status, result) = request(&router, "POST", &format!("{path}/review"), answer).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(result["memory_state"], preview["memory_state"]);
        let (status, _) = request(
            &router,
            "POST",
            &format!("{path}/review"),
            r#"{"answer":"forgot"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let (_, history) = request(&router, "GET", &format!("{path}/reviews"), "").await;
        assert_eq!(history.as_array().unwrap().len(), 2);
        assert_eq!(history[0]["rating"], 3);
        assert_eq!(history[1]["rating"], 1);
        sqlx::query("UPDATE CARDS SET due_at = 0 WHERE id = ?")
            .bind(id)
            .execute(pool.inner())
            .await
            .unwrap();
        let (status, due) = request(&router, "GET", "/card/due?limit=1", "").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(due[0]["id"], id);
        let (status, reset) = request(&router, "POST", &format!("{path}/reset-memory"), "").await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            reset["memory_state"].is_null()
                && reset["due_at"].is_null()
                && reset["last_reviewed_at"].is_null()
        );
        assert_eq!(
            request(&router, "GET", &format!("{path}/reviews"), "")
                .await
                .1
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }

    #[tokio::test]
    async fn validation_errors_and_missing_cards() {
        let (router, _pool, _dir) = setup().await;
        request(&router, "POST", "/card", &input()).await;
        for (method, path, body, status) in [
            ("GET", "/card/abc", "", 400),
            ("GET", "/card/0", "", 400),
            ("GET", "/card/list?page=0", "", 400),
            ("GET", "/card/list?page_size=201", "", 400),
            ("GET", "/card/list?page=abc", "", 400),
            ("GET", "/card/list?typo=1", "", 400),
            ("GET", "/card/due?limit=0", "", 400),
            ("GET", "/card/due?limit=201", "", 400),
            ("POST", "/card", "{", 400),
            ("POST", "/card", "{}", 400),
            ("PATCH", "/card/1/move", r#"{"deck_id":0}"#, 400),
            ("PATCH", "/card/1/move", r#"{"deck_id":3}"#, 400),
            ("PATCH", "/card/1/template", r#"{"template_id":999}"#, 400),
            (
                "PATCH",
                "/card/1/content",
                r#"{"content":{"Front":"a","Source":"changed"}}"#,
                400,
            ),
            (
                "PATCH",
                "/card/1/content",
                r#"{"content":{"Front":"a","Source":"book","Extra":1}}"#,
                400,
            ),
            ("POST", "/card/1/review", r#"{"answer":"easy"}"#, 400),
            (
                "POST",
                "/card/1/review",
                r#"{"answer":"forgot","memory_state":{}}"#,
                400,
            ),
            ("GET", "/card/999", "", 404),
            ("DELETE", "/card/999", "", 404),
            ("PATCH", "/card/999/content", r#"{"content":{}}"#, 404),
            ("PATCH", "/card/999/move", r#"{"deck_id":1}"#, 404),
            ("PATCH", "/card/999/template", r#"{"template_id":1}"#, 404),
            (
                "POST",
                "/card/999/duplicate",
                r#"{"deck_id":1,"template_id":1}"#,
                404,
            ),
            ("POST", "/card/999/review", r#"{"answer":"forgot"}"#, 404),
            (
                "POST",
                "/card/999/review/preview",
                r#"{"answer":"forgot"}"#,
                404,
            ),
            ("POST", "/card/999/reset-memory", "", 404),
            ("GET", "/card/999/reviews", "", 404),
        ] {
            let (actual, error) = request(&router, method, path, body).await;
            assert_eq!(actual.as_u16(), status, "{method} {path}: {error}");
            assert_eq!(error["success"], false);
            assert!(error["message"].is_string());
        }
    }

    #[tokio::test]
    async fn public_errors_do_not_expose_internal_identifiers() {
        let (router, _pool, _dir) = setup().await;
        request(&router, "POST", "/card", &input()).await;
        for (method, path, body, expected) in [
            ("GET", "/card/987654", "", "卡片不存在或已被删除"),
            ("GET", "/deck/987654", "", "牌组不存在或已被删除"),
            (
                "PATCH",
                "/card/1/move",
                r#"{"deck_id":987654}"#,
                "选择的牌组不存在，请重新选择",
            ),
            (
                "PATCH",
                "/card/1/template",
                r#"{"template_id":987654}"#,
                "选择的模板不存在，请重新选择",
            ),
            (
                "POST",
                "/card",
                r#"{"note_type_id":987654,"template_id":1,"deck_id":1,"content":{}}"#,
                "选择的卡片类型不存在，请重新选择",
            ),
            (
                "POST",
                "/deck",
                r#"{"name":"Test","config_id":1,"parent_id":987654}"#,
                "选择的父级文件夹不存在，请重新选择",
            ),
            (
                "POST",
                "/card/1/review",
                r#"{"answer":"987654"}"#,
                "请求数据格式不正确，请检查必填项和字段类型",
            ),
            (
                "GET",
                "/card/list?page=987654bad",
                "",
                "查询条件格式不正确，请检查分页和筛选条件",
            ),
        ] {
            let (status, error) = request(&router, method, path, body).await;
            assert!(status.is_client_error(), "{method} {path}: {error}");
            assert_eq!(error["message"], expected);
            assert!(!error.to_string().contains("987654"));
            assert!(!error.to_string().contains("_id"));
        }
    }

    #[tokio::test]
    async fn internal_errors_are_sanitized_and_docs_cover_all_routes() {
        let (router, pool, _dir) = setup().await;
        request(&router, "POST", "/card", &input()).await;
        sqlx::query("UPDATE CARDS SET last_reviewed_at = 1 WHERE id = 1")
            .execute(pool.inner())
            .await
            .unwrap();
        assert_eq!(
            request(&router, "POST", "/card/1/review", r#"{"answer":"forgot"}"#).await,
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"success":false,"message":"内部服务错误"})
            )
        );
        let (_, doc) = request(&router, "GET", "/api-docs/openapi.json", "").await;
        for (path, method) in [
            ("/card", "post"),
            ("/card/list", "get"),
            ("/card/due", "get"),
            ("/card/{id}", "get"),
            ("/card/{id}", "delete"),
            ("/card/{id}/content", "patch"),
            ("/card/{id}/move", "patch"),
            ("/card/{id}/template", "patch"),
            ("/card/{id}/duplicate", "post"),
            ("/card/{id}/reset-memory", "post"),
            ("/card/{id}/review", "post"),
            ("/card/{id}/review/preview", "post"),
            ("/card/{id}/reviews", "get"),
        ] {
            assert!(doc["paths"][path][method].is_object(), "{method} {path}");
            assert!(doc["paths"][path][method]["responses"]["500"].is_object());
        }
        assert!(doc["paths"]["/deck/list"]["get"].is_object());
        pool.inner().close().await;
        for path in ["/card/list", "/card/due", "/card/1"] {
            assert_eq!(
                request(&router, "GET", path, "").await,
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"success":false,"message":"内部服务错误"})
                )
            );
        }
    }
}
