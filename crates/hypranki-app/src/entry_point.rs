use std::path::PathBuf;

#[derive(Debug)]
pub struct LaunchOptions {
    // 项目的指定路径
    custom_project_dir: Option<PathBuf>,
}

impl LaunchOptions {
    pub fn from_envs() -> Self {
        todo!()
    }
}
