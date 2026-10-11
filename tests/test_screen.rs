use openlight::computer_use::screen::ScreenManager;

#[test]
fn test_list_monitors() {
    let sm = ScreenManager::new(1280, 1);
    let monitors = sm.list_monitors();
    assert!(!monitors.is_empty());
    let primary = &monitors[0];
    assert!(primary.width > 0);
    assert!(primary.height > 0);
}

#[test]
fn test_capture_and_scale() {
    let sm = ScreenManager::new(640, 1);
    let res = sm.capture(None);
    assert!(res.is_ok());

    let (_img, b64, scaled_dim, info) = res.unwrap();
    assert!(!b64.is_empty());
    assert!(scaled_dim.0 <= 640);
    assert!(scaled_dim.1 <= 640);

    let (real_x, real_y) = ScreenManager::scale_to_screen_coords(
        scaled_dim.0 / 2,
        scaled_dim.1 / 2,
        scaled_dim,
        &info,
    );
    assert!(real_x >= info.x);
    assert!(real_y >= info.y);
}
