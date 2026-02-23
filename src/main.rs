use floem::{kurbo::Size, window::WindowConfig, Application};
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
