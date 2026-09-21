#[cfg(target_os = "macos")]
use i_slint_backend_winit::winit::platform::macos::WindowAttributesExtMacOS;

use anyhow::{Ok, Result};
slint::include_modules!();

fn window_init() -> Result<()> {
    let mut backend = i_slint_backend_winit::Backend::new()?;

    #[cfg(target_os = "macos")]
    {
        backend.window_attributes_hook = Some(Box::new(|builder| {
            builder
                .with_fullsize_content_view(true)
                .with_titlebar_transparent(true)
                .with_title_hidden(true)
                .with_movable_by_window_background(true)
        }));
    }
    slint::platform::set_platform(Box::new(backend))?;
    Ok(())
}

pub fn entry_point() -> Result<()> {
    window_init();
    let app = AppWindow::new()?;
    app.run();
    Ok(())
}
