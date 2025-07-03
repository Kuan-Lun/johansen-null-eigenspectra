use johansen_null_eigenspectra::display_utils::{
    format_duration, format_number_with_commas, format_percentage, format_progress_bar,
    format_remaining_time,
};
use std::time::Duration;

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(Duration::from_secs(30)), "30.00 seconds");
    assert_eq!(
        format_duration(Duration::from_secs(90)),
        "1 minute 30.00 seconds"
    );
    assert_eq!(
        format_duration(Duration::from_secs(3661)),
        "1 hour 1 minute 1.00 second"
    );
    assert_eq!(
        format_duration(Duration::from_secs(90061)),
        "1 day 1 hour 1 minute 1.00 second"
    );
}

#[test]
fn test_format_number_with_commas() {
    assert_eq!(format_number_with_commas(123), "123");
    assert_eq!(format_number_with_commas(1234), "1,234");
    assert_eq!(format_number_with_commas(1234567), "1,234,567");
    assert_eq!(format_number_with_commas(1000000), "1,000,000");
}

#[test]
fn test_format_remaining_time() {
    // 測試正常情況
    let elapsed = Duration::from_secs(60);
    let remaining = format_remaining_time(elapsed, 100, 1000);
    assert!(remaining.contains("estimated remaining"));

    // 測試邊界情況
    assert_eq!(
        format_remaining_time(Duration::from_secs(60), 0, 1000),
        "unknown"
    );
    assert_eq!(
        format_remaining_time(Duration::from_secs(60), 1000, 1000),
        "unknown"
    );
}

#[test]
fn test_format_percentage() {
    assert_eq!(format_percentage(25, 100, Some(1)), "25.0%");
    assert_eq!(format_percentage(1, 3, Some(2)), "33.33%");
    assert_eq!(format_percentage(0, 100, None), "0.0%");
    assert_eq!(format_percentage(100, 0, None), "0.0%");
}

#[test]
fn test_format_progress_bar() {
    let bar = format_progress_bar(50, 100, 10);
    assert!(bar.contains("[#####     ] 50.0%"));

    let bar = format_progress_bar(0, 100, 10);
    assert!(bar.contains("[          ] 0.0%"));

    let bar = format_progress_bar(100, 100, 10);
    assert!(bar.contains("[##########] 100.0%"));
}
