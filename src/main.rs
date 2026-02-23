use floem::{kurbo::Size, text::FONT_SYSTEM, window::WindowConfig, Application};
use std::fs;
use std::io::Write;

fn main() {
    // Single-instance guard: exclusive file lock in the user's temp dir.
    // OS releases the lock automatically on process exit or crash.
    let lock_path = std::env::temp_dir().join("undone.lock");
    let lock_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&lock_path);
    let mut lock_file = match lock_file {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Undone is already running.");
            return;
        }
    };
    // Write a byte so the lock has a non-empty range.
    let _ = lock_file.write_all(b"L");
    match fs4::fs_std::FileExt::try_lock_exclusive(&lock_file) {
        Ok(true) => {}
        _ => {
            eprintln!("Undone is already running.");
            return;
        }
    }
    let _lock = lock_file; // hold until process exits

    // Register bundled Literata font (OFL-licensed, variable fonts).
    // This must happen before the Application is created so the font system
    // is ready when the first TextLayout is built.
    {
        let mut fs = FONT_SYSTEM.lock();
        let db = fs.db_mut();
        db.load_font_data(include_bytes!("../assets/fonts/Literata-Variable.ttf").to_vec());
        db.load_font_data(include_bytes!("../assets/fonts/Literata-Variable-Italic.ttf").to_vec());
    }

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
