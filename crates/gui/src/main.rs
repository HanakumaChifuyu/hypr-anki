use anyhow::Result;
pub mod windows;

fn main() -> Result<()> {
    windows::main_app_window::entry_point(|_app| {})
}
