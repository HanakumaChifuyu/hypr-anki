use super::app_error::AppError;
use crate::service::ServiceState;
use axum::{
    Json, Router,
    extract::{
        OriginalUri, Path, State,
        rejection::{JsonRejection, PathRejection},
    },
    http::StatusCode,
    routing::{get, post},
};
use hypranki_service::services::note_type::{NoteTypeError, UpdateNoteType};
use hypranki_storage::models::note::{CreateNoteType, NoteType};
use tracing::{info, warn};

#[derive(utoipa::ToSchema)]
struct NoteTypeErrorResponse {
    success: bool,
    message: String,
}

pub struct NoteTypeHandler;

impl NoteTypeHandler {
    pub fn router() -> Router<ServiceState> {
        Router::new()
            .route("/", post(create_type))
            .route("/list", get(list_types))
            .route(
                "/{id}",
                get(get_type).patch(update_type).delete(delete_type),
            )
    }

    fn service_error(error: anyhow::Error) -> AppError {
        match error.downcast_ref::<NoteTypeError>() {
            Some(NoteTypeError::TypeNotFound(_)) => {
                warn!(status = 404, error = %error, "笔记类型不存在");
                AppError::NotFound("笔记类型不存在或已被删除".into())
            }
            Some(NoteTypeError::InvalidInput(message)) => {
                warn!(status = 400, error = %error, "笔记类型校验失败");
                AppError::BadRequest(message.clone())
            }
            Some(NoteTypeError::TypeInUseForUpdate(_)) => {
                warn!(status = 400, error = %error, "笔记类型已被使用");
                AppError::BadRequest("该笔记类型已被使用，不能增删字段或修改字段名称和类型".into())
            }
            Some(NoteTypeError::TypeInUseForDelete(_)) => {
                warn!(status = 400, error = %error, "笔记类型已被使用");
                AppError::BadRequest("该笔记类型已被使用，请先删除关联的卡片或内容".into())
            }
            _ => AppError::InternalError(error),
        }
    }

    fn id(path: Result<Path<i64>, PathRejection>) -> Result<i64, AppError> {
        let Path(id) = path.map_err(|error| {
            warn!(error = %error, "笔记类型路径参数无效");
            AppError::BadRequest("无法识别要操作的笔记类型".into())
        })?;
        if id <= 0 {
            return Err(AppError::BadRequest("请选择有效的笔记类型".into()));
        }
        Ok(id)
    }

    fn body<T>(body: Result<Json<T>, JsonRejection>) -> Result<T, AppError> {
        body.map(|Json(value)| value).map_err(|error| {
            warn!(error = %error, "笔记类型请求数据格式错误");
            AppError::BadRequest("请求数据格式不正确，请检查必填项和字段类型".into())
        })
    }
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(create_type, list_types, get_type, update_type, delete_type),
tags((name = "笔记类型", description = "笔记类型与字段配置")))]
pub struct ApiDoc;

#[utoipa::path(post, path = "/note-type", tag = "笔记类型", summary = "创建笔记类型",
request_body = CreateNoteType,
responses((status = 201, description = "创建成功", body = NoteType),
(status = 400, description = "请求数据或字段定义无效", body = NoteTypeErrorResponse),
(status = 500, description = "内部服务错误", body = NoteTypeErrorResponse)))]
#[tracing::instrument(name = "创建笔记类型", skip_all, fields(method = "POST", path = %uri.path()))]
async fn create_type(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    body: Result<Json<CreateNoteType>, JsonRejection>,
) -> Result<(StatusCode, Json<NoteType>), AppError> {
    let input = NoteTypeHandler::body(body)?;
    let result = state
        .note_type_service
        .create_type(&input)
        .await
        .map_err(NoteTypeHandler::service_error)?;
    info!(status = 201, "创建笔记类型成功");
    Ok((StatusCode::CREATED, Json(result)))
}

#[utoipa::path(get, path = "/note-type/list", tag = "笔记类型", summary = "查询笔记类型列表",
responses((status = 200, description = "按创建顺序返回全部类型", body = [NoteType]),
(status = 500, description = "内部服务错误", body = NoteTypeErrorResponse)))]
#[tracing::instrument(name = "查询笔记类型列表", skip_all, fields(method = "GET", path = %uri.path()))]
async fn list_types(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
) -> Result<Json<Vec<NoteType>>, AppError> {
    let result = state
        .note_type_service
        .list_types()
        .await
        .map_err(NoteTypeHandler::service_error)?;
    info!(status = 200, count = result.len(), "查询笔记类型列表成功");
    Ok(Json(result))
}

#[utoipa::path(get, path = "/note-type/{id}", tag = "笔记类型", summary = "查询笔记类型详情",
params(("id" = i64, Path, description = "笔记类型标识", minimum = 1)),
responses((status = 200, description = "查询成功", body = NoteType),
(status = 400, description = "路径参数无效", body = NoteTypeErrorResponse),
(status = 404, description = "笔记类型不存在", body = NoteTypeErrorResponse),
(status = 500, description = "内部服务错误", body = NoteTypeErrorResponse)))]
#[tracing::instrument(name = "查询笔记类型详情", skip_all, fields(method = "GET", path = %uri.path()))]
async fn get_type(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
) -> Result<Json<NoteType>, AppError> {
    let id = NoteTypeHandler::id(path)?;
    let result = state
        .note_type_service
        .get_type(id)
        .await
        .map_err(NoteTypeHandler::service_error)?;
    info!(status = 200, "查询笔记类型详情成功");
    Ok(Json(result))
}

#[utoipa::path(patch, path = "/note-type/{id}", tag = "笔记类型", summary = "更新笔记类型",
description = "未提供的属性保持原值。fields 提供时整体替换；类型已被使用后，只能修改类型名称、字段顺序和可编辑属性。",
params(("id" = i64, Path, description = "笔记类型标识", minimum = 1)),
request_body = UpdateNoteType,
responses((status = 200, description = "更新成功", body = NoteType),
(status = 400, description = "请求无效或字段结构已被使用", body = NoteTypeErrorResponse),
(status = 404, description = "笔记类型不存在", body = NoteTypeErrorResponse),
(status = 500, description = "内部服务错误", body = NoteTypeErrorResponse)))]
#[tracing::instrument(name = "更新笔记类型", skip_all, fields(method = "PATCH", path = %uri.path()))]
async fn update_type(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
    body: Result<Json<UpdateNoteType>, JsonRejection>,
) -> Result<Json<NoteType>, AppError> {
    let id = NoteTypeHandler::id(path)?;
    let input = NoteTypeHandler::body(body)?;
    let result = state
        .note_type_service
        .update_type(id, &input)
        .await
        .map_err(NoteTypeHandler::service_error)?;
    info!(status = 200, "更新笔记类型成功");
    Ok(Json(result))
}

#[utoipa::path(delete, path = "/note-type/{id}", tag = "笔记类型", summary = "删除笔记类型",
params(("id" = i64, Path, description = "笔记类型标识", minimum = 1)),
responses((status = 204, description = "删除成功"),
(status = 400, description = "路径参数无效或类型已被使用", body = NoteTypeErrorResponse),
(status = 404, description = "笔记类型不存在", body = NoteTypeErrorResponse),
(status = 500, description = "内部服务错误", body = NoteTypeErrorResponse)))]
#[tracing::instrument(name = "删除笔记类型", skip_all, fields(method = "DELETE", path = %uri.path()))]
async fn delete_type(
    OriginalUri(uri): OriginalUri,
    State(state): State<ServiceState>,
    path: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, AppError> {
    let id = NoteTypeHandler::id(path)?;
    state
        .note_type_service
        .delete_type(id)
        .await
        .map_err(NoteTypeHandler::service_error)?;
    info!(status = 204, "删除笔记类型成功");
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use hypranki_storage::db::pool::SqlitePool;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    async fn setup() -> (Router, SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let pool = SqlitePool::new(dir.path().join("notes.db")).await.unwrap();
        pool.initialize_database().await.unwrap();
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

    fn input() -> Value {
        json!({"name":"Basic", "fields":[{"name":"Front","field_type":"Str","writability":true}]})
    }

    #[tokio::test]
    async fn type_crud_routes_and_api_docs() {
        let (router, _pool, _dir) = setup().await;
        assert_eq!(
            request(&router, "GET", "/note-type/list", "").await,
            (StatusCode::OK, json!([]))
        );
        let (status, created) = request(&router, "POST", "/note-type", &input().to_string()).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(created["name"], "Basic");
        let path = format!("/note-type/{}", created["id"]);
        assert_eq!(
            request(&router, "GET", &path, "").await,
            (StatusCode::OK, created.clone())
        );
        let (_, list) = request(&router, "GET", "/note-type/list", "").await;
        assert_eq!(list, json!([created]));
        let (status, updated) = request(&router, "PATCH", &path, r#"{"name":"Renamed"}"#).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(updated["name"], "Renamed");
        assert_eq!(updated["fields"], input()["fields"]);
        let fields =
            json!({"fields":[{"name":"Data","field_type":"Json","writability":false}]}).to_string();
        let (status, updated) = request(&router, "PATCH", &path, &fields).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(updated["name"], "Renamed");
        assert_eq!(updated["fields"][0]["field_type"], "Json");
        assert_eq!(
            request(&router, "DELETE", &path, "").await,
            (StatusCode::NO_CONTENT, Value::Null)
        );
        assert_eq!(
            request(&router, "GET", &path, "").await.0,
            StatusCode::NOT_FOUND
        );
        let (_, doc) = request(&router, "GET", "/api-docs/openapi.json", "").await;
        for (path, method) in [
            ("/note-type", "post"),
            ("/note-type/list", "get"),
            ("/note-type/{id}", "get"),
            ("/note-type/{id}", "patch"),
            ("/note-type/{id}", "delete"),
        ] {
            assert!(doc["paths"][path][method].is_object(), "{method} {path}");
            assert!(doc["paths"][path][method]["responses"]["500"].is_object());
        }
        for schema in [
            "NoteType",
            "CreateNoteType",
            "UpdateNoteType",
            "Field",
            "FieldType",
        ] {
            assert!(doc["components"]["schemas"][schema].is_object(), "{schema}");
        }
        assert!(doc["paths"]["/card/list"]["get"].is_object());
        assert!(doc["paths"]["/deck/list"]["get"].is_object());
    }

    #[tokio::test]
    async fn invalid_requests_missing_types_and_internal_errors_are_sanitized() {
        let (router, pool, _dir) = setup().await;
        for (method, path, body, expected) in [
            ("POST", "/note-type", "{", 400),
            ("POST", "/note-type", "{}", 400),
            ("POST", "/note-type", r#"{"name":"", "fields":[]}"#, 400),
            (
                "POST",
                "/note-type",
                r#"{"name":"Basic", "fields":[{"name":"Front","field_type":"Str"}]}"#,
                400,
            ),
            (
                "POST",
                "/note-type",
                r#"{"name":"Basic", "fields":[{"name":"Front","field_type":"Unknown","writability":true}]}"#,
                400,
            ),
            (
                "POST",
                "/note-type",
                r#"{"id":987654,"name":"Basic","fields":[]}"#,
                400,
            ),
            ("GET", "/note-type/987654abc", "", 400),
            ("GET", "/note-type/0", "", 400),
            ("GET", "/note-type/987654", "", 404),
            ("PATCH", "/note-type/987654", "{}", 404),
            ("DELETE", "/note-type/987654", "", 404),
            ("PATCH", "/note-type/1", r#"{"unknown":987654}"#, 400),
        ] {
            let (status, error) = request(&router, method, path, body).await;
            assert_eq!(status.as_u16(), expected, "{method} {path}: {error}");
            assert_eq!(error["success"], false);
            assert!(error["message"].is_string());
            assert!(!error.to_string().contains("987654"));
        }
        pool.inner().close().await;
        for (method, path, body) in [
            ("GET", "/note-type/list", ""),
            ("GET", "/note-type/1", ""),
            ("POST", "/note-type", input().to_string().as_str()),
            ("PATCH", "/note-type/1", "{}"),
            ("DELETE", "/note-type/1", ""),
        ] {
            assert_eq!(
                request(&router, method, path, body).await,
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"success":false,"message":"内部服务错误"})
                )
            );
        }
    }

    #[tokio::test]
    async fn referenced_types_protect_cards_and_allow_metadata_updates() {
        let (router, pool, _dir) = setup().await;
        let (_, created) = request(&router, "POST", "/note-type", &input().to_string()).await;
        let id = created["id"].as_i64().unwrap();
        let path = format!("/note-type/{id}");
        sqlx::raw_sql("INSERT INTO DECKS (id,name,config_id,created_at,updated_at) VALUES (1,'A',1,0,0); INSERT INTO TEMPLATES (id,name) VALUES (1,'A');")
            .execute(pool.inner()).await.unwrap();
        let card_input =
            json!({"note_type_id":id,"template_id":1,"deck_id":1,"content":{"Front":"hello"}});
        let (status, card) = request(&router, "POST", "/card", &card_input.to_string()).await;
        assert_eq!(status, StatusCode::CREATED);
        let (status, error) = request(&router, "DELETE", &path, "").await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            error["message"],
            "该笔记类型已被使用，请先删除关联的卡片或内容"
        );
        let schema_change = r#"{"name":"Should roll back","fields":[{"name":"Back","field_type":"Str","writability":true}]}"#;
        let (status, error) = request(&router, "PATCH", &path, schema_change).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            error["message"],
            "该笔记类型已被使用，不能增删字段或修改字段名称和类型"
        );
        assert_eq!(request(&router, "GET", &path, "").await.1["name"], "Basic");
        let metadata = r#"{"name":"Renamed","fields":[{"name":"Front","field_type":"Str","writability":false}]}"#;
        assert_eq!(
            request(&router, "PATCH", &path, metadata).await.0,
            StatusCode::OK
        );
        let card_path = format!("/card/{}", card["id"]);
        let (status, details) = request(&router, "GET", &card_path, "").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(details["card_type_name"], "Renamed");
        assert_eq!(details["fields"][0]["writability"], false);
        assert_eq!(details["content"], json!({"Front":"hello"}));
        assert_eq!(
            request(&router, "DELETE", &card_path, "").await.0,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            request(&router, "DELETE", &path, "").await.0,
            StatusCode::NO_CONTENT
        );
    }
}
