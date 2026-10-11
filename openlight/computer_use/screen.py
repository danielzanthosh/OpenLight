"""Screen capture, coordinate normalization, and visual Set-of-Marks (SoM) grounding."""

import base64
import io
from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple
import mss
from PIL import Image, ImageDraw, ImageFont


@dataclass
class ScreenInfo:
    """Information about a display monitor."""
    index: int
    left: int
    top: int
    width: int
    height: int
    is_primary: bool


class ScreenManager:
    """High-performance multi-monitor screen capture and image processing."""

    def __init__(self, monitor_index: int = 1, max_dimension: int = 1280):
        self.monitor_index = monitor_index
        self.max_dimension = max_dimension

    def list_monitors(self) -> List[ScreenInfo]:
        """List all available monitors."""
        monitors = []
        try:
            with mss.MSS() as sct:
                for idx, m in enumerate(sct.monitors):
                    if idx == 0:
                        # Index 0 is the virtual canvas containing all monitors
                        continue
                    monitors.append(
                        ScreenInfo(
                            index=idx,
                            left=m["left"],
                            top=m["top"],
                            width=m["width"],
                            height=m["height"],
                            is_primary=(idx == 1),
                        )
                    )
        except Exception:
            monitors.append(ScreenInfo(index=1, left=0, top=0, width=1920, height=1080, is_primary=True))
        return monitors

    def capture_raw(self, monitor_idx: Optional[int] = None) -> Tuple[Image.Image, ScreenInfo]:
        """Capture raw screenshot for the given monitor index with graceful fallback."""
        target_idx = monitor_idx or self.monitor_index
        try:
            with mss.MSS() as sct:
                if target_idx >= len(sct.monitors):
                    target_idx = 1
                m = sct.monitors[target_idx]
                raw_shot = sct.grab(m)
                img = Image.frombytes("RGB", raw_shot.size, raw_shot.bgra, "raw", "BGRX")
                screen_info = ScreenInfo(
                    index=target_idx,
                    left=m["left"],
                    top=m["top"],
                    width=m["width"],
                    height=m["height"],
                    is_primary=(target_idx == 1),
                )
                return img, screen_info
        except Exception:
            # Fallback for non-interactive desktop sessions, locked screens, or headless test runners
            fallback_w, fallback_h = 1920, 1080
            fallback_img = Image.new("RGB", (fallback_w, fallback_h), color=(30, 30, 40))
            draw = ImageDraw.Draw(fallback_img)
            draw.text(
                (fallback_w // 2 - 200, fallback_h // 2),
                "[OpenLight Virtual Desktop - Screen Composition Fallback]",
                fill=(200, 200, 255),
            )
            screen_info = ScreenInfo(
                index=target_idx,
                left=0,
                top=0,
                width=fallback_w,
                height=fallback_h,
                is_primary=True,
            )
            return fallback_img, screen_info

    def capture(
        self,
        monitor_idx: Optional[int] = None,
        draw_grid: bool = False,
        elements: Optional[List[Dict]] = None,
    ) -> Tuple[Image.Image, str, Tuple[int, int], ScreenInfo]:
        """
        Capture screen, optionally resize for LLM vision tokens, and return:
        (PIL Image, base64 JPEG string, (scaled_width, scaled_height), ScreenInfo).
        """
        img, screen_info = self.capture_raw(monitor_idx)

        # Optionally draw element bounding boxes or coordinate grid
        if draw_grid:
            self.draw_coordinate_grid(img)
        elif elements:
            self.draw_som_elements(img, elements)

        # Scale down if larger than max_dimension
        orig_w, orig_h = img.size
        scale = min(1.0, self.max_dimension / max(orig_w, orig_h))
        scaled_w = int(orig_w * scale)
        scaled_h = int(orig_h * scale)

        if scale < 1.0:
            resized_img = img.resize((scaled_w, scaled_h), Image.Resampling.LANCZOS)
        else:
            resized_img = img

        # Encode to JPEG base64
        buf = io.BytesIO()
        resized_img.save(buf, format="JPEG", quality=85)
        base64_str = base64.b64encode(buf.getvalue()).decode("utf-8")

        return resized_img, base64_str, (scaled_w, scaled_h), screen_info

    def scale_to_screen_coords(
        self,
        x: int,
        y: int,
        scaled_dim: Tuple[int, int],
        screen_info: ScreenInfo,
    ) -> Tuple[int, int]:
        """Convert scaled image pixel coordinates back to real desktop screen coordinates."""
        scaled_w, scaled_h = scaled_dim
        norm_x = x / scaled_w
        norm_y = y / scaled_h
        real_x = int(screen_info.left + norm_x * screen_info.width)
        real_y = int(screen_info.top + norm_y * screen_info.height)
        return real_x, real_y

    @staticmethod
    def draw_coordinate_grid(img: Image.Image, step: int = 100):
        """Draw an overlay grid with coordinate marks on the image."""
        draw = ImageDraw.Draw(img)
        w, h = img.size
        # Draw vertical lines
        for x in range(0, w, step):
            draw.line([(x, 0), (x, h)], fill=(255, 0, 0, 128), width=1)
            draw.text((x + 2, 5), str(x), fill=(255, 0, 0))

        # Draw horizontal lines
        for y in range(0, h, step):
            draw.line([(0, y), (w, y)], fill=(255, 0, 0, 128), width=1)
            draw.text((5, y + 2), str(y), fill=(255, 0, 0))

    @staticmethod
    def draw_som_elements(img: Image.Image, elements: List[Dict]):
        """Draw Set-of-Marks (SoM) bounding boxes with element index badges."""
        draw = ImageDraw.Draw(img)
        for el in elements:
            idx = el.get("index", "")
            bbox = el.get("bbox")  # (x, y, w, h)
            if not bbox or len(bbox) != 4:
                continue
            x, y, w, h = bbox
            draw.rectangle([x, y, x + w, y + h], outline=(0, 200, 255), width=2)
            # Draw badge background
            draw.rectangle([x, y - 18, x + 24, y], fill=(0, 200, 255))
            draw.text((x + 3, y - 16), f"#{idx}", fill=(0, 0, 0))
