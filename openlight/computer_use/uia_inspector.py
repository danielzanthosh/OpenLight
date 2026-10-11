"""Windows UI Automation (UIA) inspector for element tree indexing."""

import sys
from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple


@dataclass
class UIElement:
    """An interactable UI element on the desktop."""
    index: int
    control_type: str
    name: str
    bbox: Tuple[int, int, int, int]  # (left, top, width, height)
    center: Tuple[int, int]
    is_enabled: bool


class UIAInspector:
    """Inspects active application windows and UI Automation elements."""

    def __init__(self, max_elements: int = 50):
        self.max_elements = max_elements
        self.is_windows = sys.platform == "win32"
        self._uia = None
        if self.is_windows:
            try:
                import uiautomation as uia
                self._uia = uia
            except ImportError:
                self._uia = None

    def get_active_window_title(self) -> str:
        """Get the title of currently foreground/focused window."""
        if not self.is_windows:
            return "Active Window (Non-Windows)"
        try:
            import win32gui
            hwnd = win32gui.GetForegroundWindow()
            return win32gui.GetWindowText(hwnd) or "Desktop"
        except Exception:
            return "Unknown"

    def inspect_active_window(self) -> List[UIElement]:
        """
        Extract interactive UI elements from the currently focused window.
        Returns a list of UIElement objects with numbered indices #1, #2, etc.
        """
        if not self.is_windows or not self._uia:
            return []

        elements: List[UIElement] = []
        try:
            # Focused element or active window
            root = self._uia.GetForegroundControl()
            if not root:
                root = self._uia.GetRootControl()

            interactive_types = {
                "ButtonControl",
                "EditControl",
                "HyperlinkControl",
                "MenuItemControl",
                "TabItemControl",
                "CheckBoxControl",
                "RadioButtonControl",
                "ComboBoxControl",
                "ListItemControl",
                "TreeItemControl",
                "ToolBarControl",
                "CustomControl",
            }

            counter = 1

            def walk(control, depth=0):
                nonlocal counter
                if counter > self.max_elements or depth > 8:
                    return

                try:
                    rect = control.BoundingRectangle
                    if rect and rect.width() > 0 and rect.height() > 0:
                        c_type = control.ControlTypeName
                        name = (control.Name or "").strip()
                        # Include if interactive control type or has meaningful name and reasonable size
                        if c_type in interactive_types or (name and rect.width() < 1200 and rect.height() < 200):
                            center_x = rect.left + rect.width() // 2
                            center_y = rect.top + rect.height() // 2
                            elements.append(
                                UIElement(
                                    index=counter,
                                    control_type=c_type.replace("Control", ""),
                                    name=name[:60] if name else "<unnamed>",
                                    bbox=(rect.left, rect.top, rect.width(), rect.height()),
                                    center=(center_x, center_y),
                                    is_enabled=control.IsEnabled,
                                )
                            )
                            counter += 1

                    for child in control.GetChildren():
                        walk(child, depth + 1)
                except Exception:
                    pass

            walk(root)
        except Exception:
            pass

        return elements

    def format_elements_for_llm(self, elements: List[UIElement]) -> str:
        """Format element list into a clean numbered prompt block."""
        if not elements:
            return "No interactable UI elements detected via accessibility tree."

        lines = ["Detected UI Elements (Use element=<number> to interact):"]
        for el in elements:
            x, y, w, h = el.bbox
            lines.append(f"#{el.index} [{el.control_type}] \"{el.name}\" @ ({x}, {y}, {w}x{h})")
        return "\n".join(lines)
