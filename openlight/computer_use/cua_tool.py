"""Unified Computer Use Tool for LLM agent interaction."""

from typing import Any, Dict, List, Optional, Tuple
from openlight.computer_use.controller import DesktopController
from openlight.computer_use.screen import ScreenManager, ScreenInfo
from openlight.computer_use.uia_inspector import UIAInspector, UIElement


class ComputerUseTool:
    """Tool that allows an LLM agent to perceive and actuate the user's computer desktop."""

    name = "computer_use"
    description = (
        "Interact with the computer desktop, windows, mouse, and keyboard. "
        "Actions include: 'capture' (take screenshot + inspect elements), 'click', "
        "'double_click', 'right_click', 'move', 'type', 'hotkey', 'press', 'scroll', 'drag', 'wait'."
    )

    def __init__(self, monitor_index: int = 1, max_dimension: int = 1280):
        self.screen_manager = ScreenManager(monitor_index=monitor_index, max_dimension=max_dimension)
        self.uia_inspector = UIAInspector(max_elements=60)
        self.controller = DesktopController()
        self.last_screen_info: Optional[ScreenInfo] = None
        self.last_scaled_dim: Optional[Tuple[int, int]] = None
        self.last_elements: List[UIElement] = []
        self.last_screenshot_base64: Optional[str] = None

    def get_tool_schema(self) -> Dict[str, Any]:
        """Return the function declaration schema for LLM providers."""
        return {
            "name": self.name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": [
                            "capture",
                            "click",
                            "double_click",
                            "right_click",
                            "move",
                            "type",
                            "press",
                            "hotkey",
                            "scroll",
                            "drag",
                            "wait",
                        ],
                        "description": "The desktop action to perform.",
                    },
                    "element": {
                        "type": "integer",
                        "description": "The element index number (#N) detected from the latest capture to click directly.",
                    },
                    "coordinate": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": "Optional [x, y] coordinates on the screenshot image to target.",
                    },
                    "text": {
                        "type": "string",
                        "description": "Text to type for 'type' action.",
                    },
                    "key": {
                        "type": "string",
                        "description": "Single key name for 'press' action (e.g. 'enter', 'tab', 'esc').",
                    },
                    "keys": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Key sequence for 'hotkey' action (e.g. ['ctrl', 'c'], ['alt', 'tab']).",
                    },
                    "direction": {
                        "type": "string",
                        "enum": ["up", "down"],
                        "description": "Direction for 'scroll' action.",
                    },
                    "clicks": {
                        "type": "integer",
                        "description": "Number of scroll clicks/ticks or click repetitions.",
                    },
                    "start_coordinate": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": "Starting [x, y] for 'drag' action.",
                    },
                    "end_coordinate": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": "Ending [x, y] for 'drag' action.",
                    },
                    "seconds": {
                        "type": "number",
                        "description": "Seconds to pause for 'wait' action.",
                    },
                    "capture_after": {
                        "type": "boolean",
                        "description": "Whether to automatically take a new screenshot after performing this action.",
                    },
                },
                "required": ["action"],
            },
        }

    def execute(self, **kwargs) -> Dict[str, Any]:
        """Execute the computer use action and return result string + optional image."""
        action = kwargs.get("action", "capture")
        capture_after = kwargs.get("capture_after", False)
        output_message = ""
        captured_image_b64 = None

        try:
            if action == "capture":
                elements = self.uia_inspector.inspect_active_window()
                self.last_elements = elements
                el_dicts = [{"index": e.index, "bbox": e.bbox} for e in elements]

                _, b64, scaled_dim, s_info = self.screen_manager.capture(
                    draw_grid=False,
                    elements=el_dicts,
                )
                self.last_screenshot_base64 = b64
                self.last_scaled_dim = scaled_dim
                self.last_screen_info = s_info
                captured_image_b64 = b64

                window_title = self.uia_inspector.get_active_window_title()
                elements_text = self.uia_inspector.format_elements_for_llm(elements)
                output_message = (
                    f"Captured screen on Monitor {s_info.index} ({s_info.width}x{s_info.height}). "
                    f"Active Window: '{window_title}'.\n{elements_text}"
                )

            elif action in ("click", "double_click", "right_click", "move"):
                real_x, real_y = self._resolve_target(
                    element=kwargs.get("element"),
                    coord=kwargs.get("coordinate"),
                )

                if action == "click":
                    clicks = kwargs.get("clicks", 1)
                    output_message = self.controller.mouse_click(real_x, real_y, button="left", clicks=clicks)
                elif action == "double_click":
                    output_message = self.controller.mouse_double_click(real_x, real_y)
                elif action == "right_click":
                    output_message = self.controller.mouse_right_click(real_x, real_y)
                elif action == "move":
                    output_message = self.controller.mouse_move(real_x, real_y)

            elif action == "type":
                text = kwargs.get("text", "")
                output_message = self.controller.keyboard_type(text)

            elif action == "press":
                key = kwargs.get("key", "enter")
                output_message = self.controller.keyboard_press(key)

            elif action == "hotkey":
                keys = kwargs.get("keys", [])
                output_message = self.controller.keyboard_hotkey(*keys)

            elif action == "scroll":
                direction = kwargs.get("direction", "down")
                clicks = kwargs.get("clicks", 3)
                output_message = self.controller.mouse_scroll(clicks=clicks, direction=direction)

            elif action == "drag":
                start = kwargs.get("start_coordinate")
                end = kwargs.get("end_coordinate")
                if not start or not end:
                    raise ValueError("Both start_coordinate and end_coordinate required for drag.")
                start_x, start_y = self._resolve_target(coord=start)
                end_x, end_y = self._resolve_target(coord=end)
                output_message = self.controller.mouse_drag(start_x, start_y, end_x, end_y)

            elif action == "wait":
                secs = kwargs.get("seconds", 1.0)
                output_message = self.controller.wait(secs)

            else:
                output_message = f"Unknown action: '{action}'"

            # Auto re-capture if requested and not already captured
            if capture_after and action != "capture":
                _, b64, scaled_dim, s_info = self.screen_manager.capture()
                self.last_screenshot_base64 = b64
                self.last_scaled_dim = scaled_dim
                self.last_screen_info = s_info
                captured_image_b64 = b64
                output_message += "\n(Post-action screenshot captured)"

            return {
                "success": True,
                "output": output_message,
                "image_base64": captured_image_b64,
            }

        except Exception as e:
            return {
                "success": False,
                "output": f"Error executing computer_use action '{action}': {str(e)}",
                "image_base64": None,
            }

    def _resolve_target(
        self,
        element: Optional[int] = None,
        coord: Optional[List[int]] = None,
    ) -> Tuple[int, int]:
        """Resolve either an element index or coordinate array to desktop screen (x, y)."""
        if element is not None:
            for el in self.last_elements:
                if el.index == element:
                    return el.center
            raise ValueError(f"UI element #{element} not found in recent capture cache.")

        if coord is not None and len(coord) == 2:
            img_x, img_y = coord
            if self.last_scaled_dim and self.last_screen_info:
                return self.screen_manager.scale_to_screen_coords(
                    img_x,
                    img_y,
                    self.last_scaled_dim,
                    self.last_screen_info,
                )
            return img_x, img_y

        # If neither provided, use current mouse position
        return self.controller.get_mouse_position()
