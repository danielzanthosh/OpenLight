"""Virtual mouse and keyboard automation controller with safety guardrails."""

import time
from typing import List, Optional, Tuple
import pyautogui

# Set PyAutoGUI fail-safe to True by default
pyautogui.FAILSAFE = True
pyautogui.PAUSE = 0.05


class DesktopController:
    """Controls physical/virtual mouse and keyboard actions on the operating system."""

    def __init__(self, move_duration: float = 0.15, typing_interval: float = 0.015):
        self.move_duration = move_duration
        self.typing_interval = typing_interval

    def get_mouse_position(self) -> Tuple[int, int]:
        """Return the current (x, y) coordinates of the mouse cursor."""
        pos = pyautogui.position()
        return pos.x, pos.y

    def mouse_move(self, x: int, y: int, duration: Optional[float] = None) -> str:
        """Move the mouse cursor to (x, y)."""
        dur = duration if duration is not None else self.move_duration
        pyautogui.moveTo(x, y, duration=dur)
        return f"Moved mouse to ({x}, {y})"

    def mouse_click(
        self,
        x: Optional[int] = None,
        y: Optional[int] = None,
        button: str = "left",
        clicks: int = 1,
    ) -> str:
        """Click at (x, y) or the current mouse position."""
        if x is not None and y is not None:
            pyautogui.moveTo(x, y, duration=self.move_duration)
        pyautogui.click(x=x, y=y, clicks=clicks, button=button)
        btn_str = f"{clicks}x " if clicks > 1 else ""
        return f"Clicked {btn_str}{button} at ({x if x is not None else 'current'}, {y if y is not None else 'current'})"

    def mouse_double_click(self, x: Optional[int] = None, y: Optional[int] = None) -> str:
        """Double click at (x, y)."""
        return self.mouse_click(x=x, y=y, button="left", clicks=2)

    def mouse_right_click(self, x: Optional[int] = None, y: Optional[int] = None) -> str:
        """Right click at (x, y)."""
        return self.mouse_click(x=x, y=y, button="right", clicks=1)

    def mouse_drag(
        self,
        start_x: int,
        start_y: int,
        end_x: int,
        end_y: int,
        duration: float = 0.3,
        button: str = "left",
    ) -> str:
        """Drag mouse from (start_x, start_y) to (end_x, end_y)."""
        pyautogui.moveTo(start_x, start_y, duration=self.move_duration)
        pyautogui.dragTo(end_x, end_y, duration=duration, button=button)
        return f"Dragged from ({start_x}, {start_y}) to ({end_x}, {end_y})"

    def mouse_scroll(self, clicks: int, direction: str = "down", x: Optional[int] = None, y: Optional[int] = None) -> str:
        """Scroll mouse wheel up or down."""
        if x is not None and y is not None:
            pyautogui.moveTo(x, y, duration=self.move_duration)
        amount = -clicks if direction == "down" else clicks
        pyautogui.scroll(amount)
        return f"Scrolled {direction} by {clicks} ticks"

    def keyboard_type(self, text: str, interval: Optional[float] = None) -> str:
        """Type text string."""
        inter = interval if interval is not None else self.typing_interval
        # Use write for normal text; handles special characters
        pyautogui.write(text, interval=inter)
        return f"Typed {len(text)} characters"

    def keyboard_press(self, key: str) -> str:
        """Press a single key (e.g., 'enter', 'tab', 'esc', 'backspace')."""
        pyautogui.press(key.lower())
        return f"Pressed key '{key}'"

    def keyboard_hotkey(self, *keys: str) -> str:
        """Press a key combination (e.g., 'ctrl', 'c' or 'alt', 'tab')."""
        clean_keys = [k.lower().strip() for k in keys if k]
        pyautogui.hotkey(*clean_keys)
        return f"Triggered hotkey '{'+'.join(clean_keys)}'"

    def wait(self, seconds: float) -> str:
        """Pause execution for specified seconds."""
        time.sleep(seconds)
        return f"Waited {seconds}s"
