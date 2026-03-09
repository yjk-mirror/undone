use floem::{kurbo::Size, text::FONT_SYSTEM, window::WindowConfig, Application};
use std::fs;
use std::io::Write;

fn parse_cli_flags<I, S>(args: I) -> Result<(bool, bool), &'static str>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<String> = args
        .into_iter()
        .map(|arg| arg.as_ref().to_string())
        .collect();
    let dev_mode = args.iter().any(|arg| arg == "--dev");
    let quick_start = args.iter().any(|arg| arg == "--quick");

    if quick_start && !dev_mode {
        return Err("--quick requires --dev");
    }

    Ok((dev_mode, quick_start))
}

fn main() {
    let (dev_mode, quick_start) = match parse_cli_flags(std::env::args()) {
        Ok(flags) => flags,
        Err(message) => {
            eprintln!("{message}");
            return;
        }
    };

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
            move |_| undone_ui::app_view(dev_mode, quick_start),
            Some(
                WindowConfig::default()
                    .size(Size::new(1200.0, 800.0))
                    .title("Undone")
                    .show_titlebar(false),
            ),
        )
        .run();
}

#[cfg(test)]
mod tests {
    use crate::parse_cli_flags;

    #[test]
    fn parse_cli_flags_accepts_dev_and_quick_together() {
        let flags = parse_cli_flags(["undone", "--dev", "--quick"]).unwrap();
        assert_eq!(flags, (true, true));
    }

    #[test]
    fn parse_cli_flags_rejects_quick_without_dev() {
        let err = parse_cli_flags(["undone", "--quick"]).unwrap_err();
        assert_eq!(err, "--quick requires --dev");
    }
}
