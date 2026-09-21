use anyhow::Result;
pub mod windows;

fn main() {
    windows::main_app_window::entry_point();
}
