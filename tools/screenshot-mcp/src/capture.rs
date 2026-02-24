use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use windows_capture::{
    capture::{CaptureControl, Context, GraphicsCaptureApiHandler},
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    },
    window::Window,
};

// ── Frame data ─────────────────────────────────────────────────────

pub struct CapturedFrame {
    pub data: Vec<u8>, // BGRA8 pixels, no padding
    pub width: u32,
    pub height: u32,
}

type SharedFrame = Arc<Mutex<Option<CapturedFrame>>>;

// ── Persistent capture handler ─────────────────────────────────────

type CaptureError = Box<dyn std::error::Error + Send + Sync>;

struct PersistentCapture {
    shared: SharedFrame,
}

impl GraphicsCaptureApiHandler for PersistentCapture {
    type Flags = SharedFrame;
    type Error = CaptureError;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self { shared: ctx.flags })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        // Skip bad frames (can happen on first frame for some DirectX windows).
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

        // Do NOT call capture_control.stop() — session stays alive for future reads.
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// ── Session — one per captured window ──────────────────────────────

struct Session {
    shared: SharedFrame,
    control: Option<CaptureControl<PersistentCapture, CaptureError>>,
}

impl Session {
    fn start(title_fragment: &str) -> anyhow::Result<Self> {
        let window = Window::from_contains_name(title_fragment)
            .map_err(|e| anyhow::anyhow!("window '{}' not found: {}", title_fragment, e))?;

        let shared: SharedFrame = Arc::new(Mutex::new(None));

        let settings = Settings::new(
            window,
            CursorCaptureSettings::WithoutCursor,
            DrawBorderSettings::WithoutBorder,
            SecondaryWindowSettings::Default,
            // 10 fps — plenty for agent screenshots, saves CPU in persistent mode.
            MinimumUpdateIntervalSettings::Custom(Duration::from_millis(100)),
            DirtyRegionSettings::Default,
            ColorFormat::Bgra8,
            Arc::clone(&shared),
        );

        let control = PersistentCapture::start_free_threaded(settings)
            .map_err(|e| anyhow::anyhow!("capture start failed: {}", e))?;

        Ok(Self {
            shared,
            control: Some(control),
        })
    }

    /// Read the latest captured frame. Returns None if no frame has arrived yet.
    fn latest_frame(&self) -> Option<CapturedFrame> {
        self.shared.lock().unwrap().take()
    }

    /// Check if the capture thread has exited (window closed, error, etc).
    fn is_finished(&self) -> bool {
        self.control.as_ref().is_some_and(|c| c.is_finished())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Some(control) = self.control.take() {
            let _ = control.stop();
        }
    }
}

// ── SessionManager — manages sessions across MCP requests ──────────

pub struct SessionManager {
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Get the latest frame for the given window title.
    /// Creates or re-creates the capture session as needed.
    /// Waits briefly on first call for the initial frame to arrive.
    pub fn capture(&mut self, title: &str) -> anyhow::Result<CapturedFrame> {
        // Evict dead sessions (window closed, errors).
        if self
            .sessions
            .get(title)
            .is_some_and(|s| s.is_finished())
        {
            self.sessions.remove(title);
        }

        // Start session if needed.
        let is_new = !self.sessions.contains_key(title);
        if is_new {
            let session = Session::start(title)?;
            self.sessions.insert(title.to_string(), session);
        }

        let session = self.sessions.get(title).unwrap();

        // On first call, wait for the initial frame (WGC needs a GPU round-trip).
        if is_new {
            for _ in 0..50 {
                // 50 × 20ms = 1 second max wait
                if session.shared.lock().unwrap().is_some() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }

        session
            .latest_frame()
            .ok_or_else(|| anyhow::anyhow!("no frame received from '{}'", title))
    }
}
