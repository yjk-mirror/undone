use floem::{kurbo::Size, window::WindowConfig, Application};

fn main() {
    Application::new()
        .window(
            move |_| undone_ui::app_view(),
            Some(
                WindowConfig::default()
                    .size(Size::new(1200.0, 800.0))
                    .title("Undone")
                    .show_titlebar(false),
            ),
        )
        .run();
}
