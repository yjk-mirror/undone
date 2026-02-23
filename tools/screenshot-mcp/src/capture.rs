use std::sync::{Arc, Mutex};
use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    },
    window::Window,
};

/// Captured frame data shared between the capture thread and the caller.
type SharedFrame = Arc<Mutex<Option<CapturedFrame>>>;

pub struct CapturedFrame {
    pub data: Vec<u8>, // BGRA8 pixels, no padding
    pub width: u32,
    pub height: u32,
}

struct OneShotCapture {
    shared: SharedFrame,
}

impl GraphicsCaptureApiHandler for OneShotCapture {
    type Flags = SharedFrame;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self { shared: ctx.flags })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        // frame.buffer() can fail on the first frame for some DirectX windows.
        // Skip bad frames rather than aborting — a good frame usually follows.
        let Ok(mut buf) = frame.buffer() else {
            return Ok(());
        };
        let width = buf.width();
        let height = buf.height();

        let data = if buf.has_padding() {
            buf.as_nopadding_buffer()?.to_vec()
        } else {
            buf.as_raw_buffer().to_vec()
        };

        *self.shared.lock().unwrap() = Some(CapturedFrame { data, width, height });

        // Take exactly one frame and stop — no continuous capture loop.
        capture_control.stop();
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Capture a single frame from the window whose title contains `title_fragment`.
/// No focus stealing. No border on Windows 11. Cursor excluded.
/// Returns raw BGRA8 pixel bytes plus dimensions.
pub fn capture_window(title_fragment: &str) -> anyhow::Result<CapturedFrame> {
    let window = Window::from_contains_name(title_fragment)
        .map_err(|e| anyhow::anyhow!("window '{}' not found: {}", title_fragment, e))?;

    let shared: SharedFrame = Arc::new(Mutex::new(None));

    let settings = Settings::new(
        window,
        CursorCaptureSettings::WithoutCursor,
        DrawBorderSettings::WithoutBorder,
        SecondaryWindowSettings::Default,
        MinimumUpdateIntervalSettings::Default,
        DirtyRegionSettings::Default,
        ColorFormat::Bgra8,
        Arc::clone(&shared),
    );

    // Spawns a capture thread; does not activate or focus the window.
    let control = OneShotCapture::start_free_threaded(settings)
        .map_err(|e| anyhow::anyhow!("capture start failed: {}", e))?;

    // Wait for on_frame_arrived to call capture_control.stop() and finish naturally.
    // Do NOT use control.stop() here — that sends WM_QUIT which kills the thread
    // before the first frame arrives, leaving shared=None.
    control
        .wait()
        .map_err(|e| anyhow::anyhow!("capture wait failed: {}", e))?;

    let frame = shared.lock().unwrap().take();
    frame.ok_or_else(|| anyhow::anyhow!("no frame received"))
}
