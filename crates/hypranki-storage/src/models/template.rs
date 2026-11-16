use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 独立存储的模板基础信息，卡片通过 template_id 引用。
/// 模板渲染内容待确定格式后扩展。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema, FromRow)]
pub struct Template {
    pub id: i64,
    pub name: String,
}
