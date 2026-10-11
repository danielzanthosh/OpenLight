pub struct UIAInspector;

impl UIAInspector {
    pub fn get_active_window_title() -> String {
        #[cfg(windows)]
        {
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};
                let hwnd = GetForegroundWindow();
                let mut buf = [0u16; 512];
                let len = GetWindowTextW(hwnd, &mut buf);
                if len > 0 {
                    return String::from_utf16_lossy(&buf[..len as usize]);
                }
            }
        }
        "Desktop".to_string()
    }
}
