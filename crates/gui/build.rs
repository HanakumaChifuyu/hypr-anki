fn main() {
    unsafe {
        std::env::set_var("SLINT_ENABLE_EXPERIMENTAL_FEATURES", "1");
    }
    slint_build::compile("ui/appwindow.slint").unwrap();
}
