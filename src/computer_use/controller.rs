use std::thread::sleep;
use std::time::Duration;

pub struct DesktopController {
    pub move_duration_ms: u64,
}

impl Default for DesktopController {
    fn default() -> Self {
        Self {
            move_duration_ms: 50,
        }
    }
}

impl DesktopController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mouse_move(&self, x: i32, y: i32) -> Result<String, String> {
        self.check_failsafe(x, y)?;
        #[cfg(windows)]
        {
            unsafe {
                use windows::Win32::UI::Input::KeyboardAndMouse::{
                    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_MOVE,
                    MOUSEINPUT,
                };
                use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

                let screen_w = GetSystemMetrics(SM_CXSCREEN) as f32;
                let screen_h = GetSystemMetrics(SM_CYSCREEN) as f32;

                // SendInput absolute coordinates are mapped from 0 to 65535
                let norm_x = ((x as f32 / screen_w) * 65535.0) as i32;
                let norm_y = ((y as f32 / screen_h) * 65535.0) as i32;

                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: norm_x,
                            dy: norm_y,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
        }
        sleep(Duration::from_millis(self.move_duration_ms));
        Ok(format!("Moved mouse to ({x}, {y})"))
    }

    pub fn mouse_click(&self, x: Option<i32>, y: Option<i32>, button: &str, clicks: u32) -> Result<String, String> {
        if let (Some(px), Some(py)) = (x, y) {
            self.mouse_move(px, py)?;
        }

        #[cfg(windows)]
        {
            unsafe {
                use windows::Win32::UI::Input::KeyboardAndMouse::{
                    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
                    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT,
                };

                let (down_flag, up_flag) = match button.to_lowercase().as_str() {
                    "right" => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
                    _ => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
                };

                for _ in 0..clicks {
                    let down = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: 0,
                                dwFlags: down_flag,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    let up = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: 0,
                                dwFlags: up_flag,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    SendInput(&[down, up], std::mem::size_of::<INPUT>() as i32);
                    sleep(Duration::from_millis(30));
                }
            }
        }

        Ok(format!("Clicked {button} button {clicks} time(s)"))
    }

    pub fn mouse_scroll(&self, clicks: i32, direction: &str) -> Result<String, String> {
        let delta = if direction.eq_ignore_ascii_case("up") {
            clicks.abs() * 120
        } else {
            -clicks.abs() * 120
        };

        #[cfg(windows)]
        {
            unsafe {
                use windows::Win32::UI::Input::KeyboardAndMouse::{
                    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_WHEEL, MOUSEINPUT,
                };
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: delta as u32,
                            dwFlags: MOUSEEVENTF_WHEEL,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
        }

        Ok(format!("Scrolled {direction} by {clicks} tick(s)"))
    }

    pub fn keyboard_type(&self, text: &str) -> Result<String, String> {
        #[cfg(windows)]
        {
            unsafe {
                use windows::Win32::UI::Input::KeyboardAndMouse::{
                    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
                    KEYEVENTF_UNICODE,
                };

                for c in text.encode_utf16() {
                    let down = INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(0),
                                wScan: c,
                                dwFlags: KEYEVENTF_UNICODE,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    let up = INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(0),
                                wScan: c,
                                dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    SendInput(&[down, up], std::mem::size_of::<INPUT>() as i32);
                    sleep(Duration::from_millis(15));
                }
            }
        }

        Ok(format!("Typed {} character(s)", text.len()))
    }

    pub fn keyboard_press(&self, key: &str) -> Result<String, String> {
        #[cfg(windows)]
        {
            unsafe {
                use windows::Win32::UI::Input::KeyboardAndMouse::*;

                let vk = match key.to_lowercase().as_str() {
                    "enter" | "return" => VK_RETURN,
                    "tab" => VK_TAB,
                    "escape" | "esc" => VK_ESCAPE,
                    "backspace" => VK_BACK,
                    "space" => VK_SPACE,
                    "up" => VK_UP,
                    "down" => VK_DOWN,
                    "left" => VK_LEFT,
                    "right" => VK_RIGHT,
                    _ => VK_RETURN,
                };

                let down = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: vk,
                            wScan: 0,
                            dwFlags: KEYBD_EVENT_FLAGS(0),
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                let up = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: vk,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                SendInput(&[down, up], std::mem::size_of::<INPUT>() as i32);
            }
        }

        Ok(format!("Pressed key '{key}'"))
    }

    fn check_failsafe(&self, x: i32, y: i32) -> Result<(), String> {
        if x <= 5 && y <= 5 {
            Err("Fail-safe triggered: mouse moved into upper-left corner! Execution stopped.".to_string())
        } else {
            Ok(())
        }
    }
}
