"""Tests for screen capture and coordinate scaling."""

from openlight.computer_use.screen import ScreenManager, ScreenInfo


def test_list_monitors():
    sm = ScreenManager()
    monitors = sm.list_monitors()
    assert len(monitors) >= 1
    primary = monitors[0]
    assert primary.width > 0
    assert primary.height > 0


def test_capture_and_scale():
    sm = ScreenManager(max_dimension=640)
    img, b64_str, scaled_dim, info = sm.capture()

    assert img is not None
    assert len(b64_str) > 100
    assert scaled_dim[0] <= 640
    assert scaled_dim[1] <= 640

    # Test coordinate mapping
    real_x, real_y = sm.scale_to_screen_coords(
        x=scaled_dim[0] // 2,
        y=scaled_dim[1] // 2,
        scaled_dim=scaled_dim,
        screen_info=info,
    )
    assert info.left <= real_x <= info.left + info.width
    assert info.top <= real_y <= info.top + info.height
