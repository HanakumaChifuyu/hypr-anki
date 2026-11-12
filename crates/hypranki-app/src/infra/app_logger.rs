use std::path::Path;

use tracing_appender::non_blocking::{self, WorkerGuard};
use tracing_appender::rolling::daily;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, registry};

#[derive(Debug)]
pub struct AppLogger {
    inner_guard: WorkerGuard,
}

impl AppLogger {
    pub fn new(
        log_dir: impl AsRef<Path>,
        log_prefix: impl AsRef<Path>,
        filter_env: Option<&str>,
    ) -> Self {
        let inner_guard = AppLogger::init_log(log_dir, log_prefix, filter_env);
        Self { inner_guard }
    }

    fn init_log(
        log_dir: impl AsRef<Path>,
        log_prefix: impl AsRef<Path>,
        filter_env: Option<&str>,
    ) -> WorkerGuard {
        let file_appender = daily(log_dir, log_prefix);
        let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

        let filter = EnvFilter::new(filter_env.unwrap_or("info"));

        registry()
            .with(filter)
            .with(layer().pretty())
            .with(layer().with_writer(file_writer).with_ansi(false))
            .init();
        guard
    }
}
