use std::borrow::Cow;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use tracing_subscriber::EnvFilter;

use super::app_path::AppPath;

/// 应用启动选项，由基础设施组装时使用。
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppOptions {
    /// HTTP 服务监听地址。
    pub listen_addr: SocketAddr,
    /// tracing 日志过滤表达式，支持按模块配置。
    pub log_filter: String,
    /// 是否启用 Scalar 页面和 OpenAPI JSON。
    pub docs_enabled: bool,
}

impl Default for AppOptions {
    fn default() -> Self {
        Self {
            listen_addr: SocketAddr::from((Ipv4Addr::LOCALHOST, 3000)),
            log_filter: "info".into(),
            docs_enabled: true,
        }
    }
}

impl AppOptions {
    /// 从指定文件或 AppPath 配置目录下的 config.toml 读取，失败时使用默认配置。
    pub fn load(custome_config_path: Option<&Path>, AppPath { config_dir, .. }: &AppPath) -> Self {
        let config_file: Cow<'_, Path> = custome_config_path
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(config_dir.join("config.toml")));

        Self::load_from_file(config_file).unwrap_or_else(|error| {
            tracing::warn!(error = %format!("{error:#}"), "加载应用配置失败，使用默认配置");
            Self::default()
        })
    }

    /// 从指定 TOML 文件创建选项，缺失字段使用默认值。
    /// 相对配置文件路径基于当前工作目录；不修改文件或环境变量。
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        let options: Self = config::Config::builder()
            .add_source(config::File::from(path.as_path()))
            .build()
            .with_context(|| format!("读取配置文件失败：{}", path.display()))?
            .try_deserialize()
            .with_context(|| format!("解析配置文件失败：{}", path.display()))?;

        EnvFilter::try_new(&options.log_filter).context("log_filter 不是有效的日志过滤表达式")?;
        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_uses_app_path_and_custom_file_with_default_fallback() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let app_path = AppPath {
            config_dir: dir.path().join("config"),
            cache_dir: dir.path().join("cache"),
            data_dir: dir.path().join("data"),
        };
        std::fs::create_dir_all(&app_path.config_dir)?;
        std::fs::write(
            app_path.config_dir.join("config.toml"),
            "docs_enabled = false",
        )?;
        assert!(!AppOptions::load(None, &app_path).docs_enabled);

        let custom = dir.path().join("custom.toml");
        std::fs::write(&custom, "listen_addr = '127.0.0.1:5000'")?;
        let options = AppOptions::load(Some(&custom), &app_path);
        assert_eq!(options.listen_addr.port(), 5000);
        assert!(options.docs_enabled);

        for path in [dir.path().join("missing.toml"), custom] {
            if path.exists() {
                std::fs::write(&path, "listen_addr = 'invalid'")?;
            }
            let options = AppOptions::load(Some(&path), &app_path);
            assert_eq!(options.listen_addr.to_string(), "127.0.0.1:3000");
            assert_eq!(options.log_filter, "info");
            assert!(options.docs_enabled);
        }
        Ok(())
    }

    #[test]
    fn loads_options_and_preserves_defaults() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "docs_enabled = false")?;
        let options = AppOptions::load_from_file(&path)?;
        assert!(!options.docs_enabled);
        assert_eq!(options.listen_addr.to_string(), "127.0.0.1:3000");
        assert_eq!(options.log_filter, "info");

        std::fs::write(
            &path,
            "listen_addr = '[::1]:4000'\nlog_filter = 'warn,hypranki_service=debug'\ndocs_enabled = true",
        )?;
        let options = AppOptions::load_from_file(&path)?;
        assert_eq!(options.listen_addr.to_string(), "[::1]:4000");
        assert_eq!(options.log_filter, "warn,hypranki_service=debug");
        assert!(options.docs_enabled);
        Ok(())
    }

    #[test]
    fn rejects_missing_or_invalid_files() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("config.toml");
        assert!(AppOptions::load_from_file(&path).is_err());
        for text in [
            "bad = [",
            "listen_addr = 'bad'",
            "log_filter = '[bad'",
            "docs_enabeld = false",
            "data_dir = ''",
        ] {
            std::fs::write(&path, text)?;
            assert!(AppOptions::load_from_file(&path).is_err(), "{text}");
        }
        Ok(())
    }
}
