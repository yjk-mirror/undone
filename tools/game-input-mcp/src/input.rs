use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{MapVirtualKeyW, MAPVK_VK_TO_VSC, VIRTUAL_KEY},
        WindowsAndMessaging::{
            EnumWindows, GetWindowTextLengthW, GetWindowTextW, PostMessageW, WM_KEYDOWN,
            WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL,
        },
    },
};

// ── Window finding ──────────────────────────────────────────────────

struct FindCtx {
    fragment: String,
    result: Option<HWND>,
}

unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let ctx = unsafe { &mut *(lparam.0 as *mut FindCtx) };

    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len == 0 {
        return BOOL(1); // keep enumerating
    }

    let mut buf = vec![0u16; (len + 1) as usize];
    let actual = unsafe { GetWindowTextW(hwnd, &mut buf) } as usize;
    let title = String::from_utf16_lossy(&buf[..actual]);

    if title.contains(&ctx.fragment) {
        ctx.result = Some(hwnd);
        BOOL(0) // stop
    } else {
        BOOL(1) // keep enumerating
    }
}

/// Find an HWND whose title contains the given substring (case-sensitive).
pub fn find_window(title_fragment: &str) -> anyhow::Result<HWND> {
    let mut ctx = FindCtx {
        fragment: title_fragment.to_string(),
        result: None,
    };

    unsafe {
        let _ = EnumWindows(Some(enum_callback), LPARAM(&mut ctx as *mut FindCtx as isize));
    }

    ctx.result
        .ok_or_else(|| anyhow::anyhow!("window '{}' not found", title_fragment))
}

// ── Key mapping ─────────────────────────────────────────────────────

/// Map a key name string to a VIRTUAL_KEY code.
pub fn key_to_vk(key: &str) -> anyhow::Result<VIRTUAL_KEY> {
    match key.to_lowercase().as_str() {
        "1" => Ok(VIRTUAL_KEY(0x31)),
        "2" => Ok(VIRTUAL_KEY(0x32)),
        "3" => Ok(VIRTUAL_KEY(0x33)),
        "4" => Ok(VIRTUAL_KEY(0x34)),
        "5" => Ok(VIRTUAL_KEY(0x35)),
        "6" => Ok(VIRTUAL_KEY(0x36)),
        "7" => Ok(VIRTUAL_KEY(0x37)),
        "8" => Ok(VIRTUAL_KEY(0x38)),
        "9" => Ok(VIRTUAL_KEY(0x39)),
        "enter" | "return" => Ok(VIRTUAL_KEY(0x0D)),  // VK_RETURN
        "tab" => Ok(VIRTUAL_KEY(0x09)),                // VK_TAB
        "escape" | "esc" => Ok(VIRTUAL_KEY(0x1B)),     // VK_ESCAPE
        "space" => Ok(VIRTUAL_KEY(0x20)),              // VK_SPACE
        _ => Err(anyhow::anyhow!("unsupported key: '{}'. Supported: 1-9, enter, tab, escape, space", key)),
    }
}

/// Build the lparam for WM_KEYDOWN: repeat=1, scancode from MapVirtualKeyW, no extended flag.
fn make_keydown_lparam(vk: VIRTUAL_KEY) -> LPARAM {
    let scan = unsafe { MapVirtualKeyW(vk.0 as u32, MAPVK_VK_TO_VSC) };
    // bits 0-15: repeat count (1)
    // bits 16-23: scan code
    // bit 24: extended key flag (0 for these keys)
    // bit 30: previous key state (0 = was up)
    // bit 31: transition state (0 = being pressed)
    LPARAM((1 | ((scan & 0xFF) << 16)) as isize)
}

/// Build the lparam for WM_KEYUP: repeat=1, scancode, transition bit set.
fn make_keyup_lparam(vk: VIRTUAL_KEY) -> LPARAM {
    let scan = unsafe { MapVirtualKeyW(vk.0 as u32, MAPVK_VK_TO_VSC) };
    // bit 30: previous key state (1 = was down)
    // bit 31: transition state (1 = being released)
    LPARAM((1 | ((scan & 0xFF) << 16) | (1 << 30) | (1 << 31)) as isize)
}

// ── Public actions ──────────────────────────────────────────────────

/// Post a key press (down + up) to the target window. No focus steal.
pub fn press_key(hwnd: HWND, key: &str) -> anyhow::Result<()> {
    let vk = key_to_vk(key)?;
    let wparam = WPARAM(vk.0 as usize);

    unsafe {
        PostMessageW(hwnd, WM_KEYDOWN, wparam, make_keydown_lparam(vk))
            .map_err(|e| anyhow::anyhow!("PostMessage WM_KEYDOWN failed: {}", e))?;
        PostMessageW(hwnd, WM_KEYUP, wparam, make_keyup_lparam(vk))
            .map_err(|e| anyhow::anyhow!("PostMessage WM_KEYUP failed: {}", e))?;
    }

    Ok(())
}

/// Post a mouse click (down + up) at client-relative (x, y) to the target window.
/// No focus steal, no cursor movement.
pub fn click(hwnd: HWND, x: i32, y: i32) -> anyhow::Result<()> {
    // MAKELPARAM(x, y) = (y << 16) | (x & 0xFFFF)
    let lparam = LPARAM(((y as isize) << 16) | (x as isize & 0xFFFF));

    unsafe {
        PostMessageW(hwnd, WM_LBUTTONDOWN, WPARAM(0), lparam)
            .map_err(|e| anyhow::anyhow!("PostMessage WM_LBUTTONDOWN failed: {}", e))?;
        PostMessageW(hwnd, WM_LBUTTONUP, WPARAM(0), lparam)
            .map_err(|e| anyhow::anyhow!("PostMessage WM_LBUTTONUP failed: {}", e))?;
    }

    Ok(())
}

/// Post a mouse wheel scroll at client-relative (x, y).
/// `delta` is in wheel ticks: positive = scroll up, negative = scroll down.
/// One tick is typically WHEEL_DELTA (120 units).
pub fn scroll(hwnd: HWND, x: i32, y: i32, delta: i32) -> anyhow::Result<()> {
    // WM_MOUSEWHEEL wparam: high word = wheel delta, low word = key state (0)
    let wheel_delta = delta * 120; // WHEEL_DELTA = 120
    let wparam = WPARAM(((wheel_delta as u16 as usize) << 16) | 0);
    // WM_MOUSEWHEEL lparam: screen coordinates (not client), but PostMessage
    // delivers whatever we put here — the app may or may not use it.
    let lparam = LPARAM(((y as isize) << 16) | (x as isize & 0xFFFF));

    unsafe {
        PostMessageW(hwnd, WM_MOUSEWHEEL, wparam, lparam)
            .map_err(|e| anyhow::anyhow!("PostMessage WM_MOUSEWHEEL failed: {}", e))?;
    }

    Ok(())
}

/// Post a mouse move to client-relative (x, y). Triggers hover effects.
pub fn hover(hwnd: HWND, x: i32, y: i32) -> anyhow::Result<()> {
    let lparam = LPARAM(((y as isize) << 16) | (x as isize & 0xFFFF));

    unsafe {
        PostMessageW(hwnd, WM_MOUSEMOVE, WPARAM(0), lparam)
            .map_err(|e| anyhow::anyhow!("PostMessage WM_MOUSEMOVE failed: {}", e))?;
    }

    Ok(())
}
