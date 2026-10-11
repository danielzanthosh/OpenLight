pub mod controller;
pub mod screen;
pub mod uia;

use crate::tools::{Tool, ToolResult};
use async_trait::async_trait;
use controller::DesktopController;
use screen::{ScreenInfo, ScreenManager};
use serde_json::{json, Value};
use std::sync::Mutex;
use uia::UIAInspector;

pub struct ComputerUseTool {
    screen_manager: ScreenManager,
    controller: DesktopController,
    last_scaled_dim: Mutex<Option<(u32, u32)>>,
    last_screen_info: Mutex<Option<ScreenInfo>>,
}

impl ComputerUseTool {
    pub fn new(max_dimension: u32, default_monitor: usize) -> Self {
        Self {
            screen_manager: ScreenManager::new(max_dimension, default_monitor),
            controller: DesktopController::new(),
            last_scaled_dim: Mutex::new(None),
            last_screen_info: Mutex::new(None),
        }
    }
}

#[async_trait]
impl Tool for ComputerUseTool {
    fn name(&self) -> &str {
        "computer_use"
    }

    fn description(&self) -> &str {
        "Control desktop, mouse, keyboard, and screen. Actions: capture, click, double_click, right_click, move, type, press, scroll, wait."
    }

    fn schema(&self) -> Value {
        json!({
            "name": self.name(),
            "description": self.description(),
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["capture", "click", "double_click", "right_click", "move", "type", "press", "scroll", "wait"],
                        "description": "Desktop action to execute."
                    },
                    "coordinate": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "description": "Optional [x, y] coordinates on the screenshot image to target."
                    },
                    "text": {
                        "type": "string",
                        "description": "Text to type."
                    },
                    "key": {
                        "type": "string",
                        "description": "Key to press (e.g. 'enter', 'tab', 'esc')."
                    },
                    "direction": {
                        "type": "string",
                        "enum": ["up", "down"],
                        "description": "Scroll direction."
                    },
                    "clicks": {
                        "type": "integer",
                        "description": "Number of clicks or scroll ticks."
                    },
                    "seconds": {
                        "type": "number",
                        "description": "Seconds to wait."
                    }
                },
                "required": ["action"]
            }
        })
    }

    async fn execute(&self, arguments: Value) -> ToolResult {
        let action = arguments["action"].as_str().unwrap_or("capture");

        match action {
            "capture" => {
                match self.screen_manager.capture(None) {
                    Ok((_img, b64, scaled_dim, info)) => {
                        *self.last_scaled_dim.lock().unwrap() = Some(scaled_dim);
                        *self.last_screen_info.lock().unwrap() = Some(info.clone());

                        let title = UIAInspector::get_active_window_title();
                        let msg = format!(
                            "Captured screen on Monitor {} ({}x{}). Active window: '{}'.",
                            info.index, info.width, info.height, title
                        );
                        ToolResult::with_image(msg, b64)
                    }
                    Err(e) => ToolResult::err(format!("Screen capture error: {e}")),
                }
            }
            "click" => {
                let (x, y) = self.resolve_coords(&arguments["coordinate"]);
                let clicks = arguments["clicks"].as_u64().unwrap_or(1) as u32;
                match self.controller.mouse_click(x, y, "left", clicks) {
                    Ok(msg) => ToolResult::ok(msg),
                    Err(e) => ToolResult::err(e),
                }
            }
            "double_click" => {
                let (x, y) = self.resolve_coords(&arguments["coordinate"]);
                match self.controller.mouse_click(x, y, "left", 2) {
                    Ok(msg) => ToolResult::ok(msg),
                    Err(e) => ToolResult::err(e),
                }
            }
            "right_click" => {
                let (x, y) = self.resolve_coords(&arguments["coordinate"]);
                match self.controller.mouse_click(x, y, "right", 1) {
                    Ok(msg) => ToolResult::ok(msg),
                    Err(e) => ToolResult::err(e),
                }
            }
            "move" => {
                if let (Some(x), Some(y)) = self.resolve_coords(&arguments["coordinate"]) {
                    match self.controller.mouse_move(x, y) {
                        Ok(msg) => ToolResult::ok(msg),
                        Err(e) => ToolResult::err(e),
                    }
                } else {
                    ToolResult::err("Missing coordinate for 'move' action.")
                }
            }
            "type" => {
                let text = arguments["text"].as_str().unwrap_or("");
                match self.controller.keyboard_type(text) {
                    Ok(msg) => ToolResult::ok(msg),
                    Err(e) => ToolResult::err(e),
                }
            }
            "press" => {
                let key = arguments["key"].as_str().unwrap_or("enter");
                match self.controller.keyboard_press(key) {
                    Ok(msg) => ToolResult::ok(msg),
                    Err(e) => ToolResult::err(e),
                }
            }
            "scroll" => {
                let direction = arguments["direction"].as_str().unwrap_or("down");
                let clicks = arguments["clicks"].as_i64().unwrap_or(3) as i32;
                match self.controller.mouse_scroll(clicks, direction) {
                    Ok(msg) => ToolResult::ok(msg),
                    Err(e) => ToolResult::err(e),
                }
            }
            "wait" => {
                let secs = arguments["seconds"].as_f64().unwrap_or(1.0);
                tokio::time::sleep(tokio::time::Duration::from_millis((secs * 1000.0) as u64)).await;
                ToolResult::ok(format!("Waited {secs}s"))
            }
            other => ToolResult::err(format!("Unknown computer_use action: '{other}'")),
        }
    }
}

impl ComputerUseTool {
    fn resolve_coords(&self, coord_val: &Value) -> (Option<i32>, Option<i32>) {
        if let Some(arr) = coord_val.as_array() {
            if arr.len() >= 2 {
                let ix = arr[0].as_u64().unwrap_or(0) as u32;
                let iy = arr[1].as_u64().unwrap_or(0) as u32;
                let dim_lock = self.last_scaled_dim.lock().unwrap();
                let info_lock = self.last_screen_info.lock().unwrap();
                if let (Some(scaled_dim), Some(ref info)) = (*dim_lock, info_lock.as_ref()) {
                    let (rx, ry) = ScreenManager::scale_to_screen_coords(ix, iy, scaled_dim, info);
                    return (Some(rx), Some(ry));
                }
                return (Some(ix as i32), Some(iy as i32));
            }
        }
        (None, None)
    }
}
