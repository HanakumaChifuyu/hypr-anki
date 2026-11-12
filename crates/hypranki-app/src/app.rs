use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::handlers::AppHandler;
use crate::infra::InfraState;
use crate::repo::Repositories;
use crate::service::ServiceState;

pub trait FromRef<T> {
    fn from_ref(input: &T) -> Self;
}

impl<T: Clone> FromRef<T> for T {
    fn from_ref(input: &T) -> Self {
        input.clone()
    }
}

#[derive(Debug)]
pub struct AppState {
    infra_state: InfraState,
}

impl AppState {
    pub async fn new(custom_project: Option<PathBuf>) -> Result<Self> {
        let infra_state = InfraState::new(custom_project).await?;
        infra_state
            .sqlite_pool()
            .initialize_database()
            .await
            .context("初始化数据库失败")
            .inspect_err(
                |error| tracing::error!(error = %format!("{error:#}"), "应用初始化失败"),
            )?;
        Ok(Self { infra_state })
    }

    /// 组装请求依赖并运行服务器，退出后释放基础设施资源。
    pub async fn run(self, listen_addr: impl AsRef<str>) -> Result<()> {
        let repositories = Repositories::new(&self.infra_state);
        let services = ServiceState::new(repositories);
        let result = AppHandler::launcher(listen_addr, services).await;
        if let Err(error) = &result {
            tracing::error!(error = %format!("{error:#}"), "应用服务器运行失败");
        }
        // 日志守卫必须存活到服务器退出，包括启动失败的情况。
        drop(self);
        result
    }
}
