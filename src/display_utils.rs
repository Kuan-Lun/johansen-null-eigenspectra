//! 顯示工具模組
//!
//! 提供各種用於格式化顯示的實用函數，包括時間格式化、數字格式化等。

use std::time::Duration;

/// 條件性輸出宏，根據 quiet 參數決定是否輸出
///
/// # 參數
/// * `quiet` - 是否安靜模式（true 時不輸出）
/// * `fmt` - 格式化字串
/// * `args` - 格式化參數
///
/// # 範例
/// ```
/// use johansen_null_eigenspectra::conditional_println;
///
/// let quiet = false;
/// conditional_println!(quiet, "Hello, {}!", "world"); // 會輸出
///
/// let quiet = true;
/// conditional_println!(quiet, "Hello, {}!", "world"); // 不會輸出
/// ```
#[macro_export]
macro_rules! conditional_println {
    ($quiet:expr, $($arg:tt)*) => {
        if !$quiet {
            println!($($arg)*);
        }
    };
}

/// 條件性空行輸出宏，根據 quiet 參數決定是否輸出空行
///
/// # 參數
/// * `quiet` - 是否安靜模式（true 時不輸出）
///
/// # 範例
/// ```
/// use johansen_null_eigenspectra::conditional_println_empty;
///
/// let quiet = false;
/// conditional_println_empty!(quiet); // 會輸出空行
///
/// let quiet = true;
/// conditional_println_empty!(quiet); // 不會輸出空行
/// ```
#[macro_export]
macro_rules! conditional_println_empty {
    ($quiet:expr) => {
        if !$quiet {
            println!();
        }
    };
}

/// 格式化持續時間為人類可讀的字串
///
/// # 參數
/// * `duration` - 要格式化的持續時間
///
/// # 返回值
/// 格式化後的時間字串，例如 "2 days 3 hours 45 minutes 30.25 seconds"
///
/// # 範例
/// ```text
/// use std::time::Duration;
///
/// let duration = Duration::from_secs(3725); // 1 小時 2 分鐘 5 秒
/// let formatted = format_duration(duration);
/// // 輸出: "1 hour 2 minutes 5.00 seconds"
/// ```
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs_f64();
    let days = (total_seconds / 86400.0) as u64;
    let hours = ((total_seconds % 86400.0) / 3600.0) as u64;
    let minutes = ((total_seconds % 3600.0) / 60.0) as u64;
    let seconds = total_seconds % 60.0;

    if days > 0 {
        format!(
            "{} day{} {} hour{} {} minute{} {:.2} second{}",
            days,
            if days == 1 { "" } else { "s" },
            hours,
            if hours == 1 { "" } else { "s" },
            minutes,
            if minutes == 1 { "" } else { "s" },
            seconds,
            if seconds == 1.0 { "" } else { "s" }
        )
    } else if hours > 0 {
        format!(
            "{} hour{} {} minute{} {:.2} second{}",
            hours,
            if hours == 1 { "" } else { "s" },
            minutes,
            if minutes == 1 { "" } else { "s" },
            seconds,
            if seconds == 1.0 { "" } else { "s" }
        )
    } else if minutes > 0 {
        format!(
            "{} minute{} {:.2} second{}",
            minutes,
            if minutes == 1 { "" } else { "s" },
            seconds,
            if seconds == 1.0 { "" } else { "s" }
        )
    } else {
        format!(
            "{:.2} second{}",
            seconds,
            if seconds == 1.0 { "" } else { "s" }
        )
    }
}

/// 格式化數字，添加千位分隔符
///
/// # 參數
/// * `n` - 要格式化的數字
///
/// # 返回值
/// 帶有逗號分隔符的數字字串
///
/// # 範例
/// ```text
/// assert_eq!(format_number_with_commas(1234567), "1,234,567");
/// assert_eq!(format_number_with_commas(1000), "1,000");
/// assert_eq!(format_number_with_commas(123), "123");
/// ```
pub fn format_number_with_commas(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }

    result
}

/// 格式化剩餘時間估算
///
/// # 參數
/// * `elapsed` - 已經過的時間
/// * `completed` - 已完成的工作量
/// * `total` - 總工作量
///
/// # 返回值
/// 剩餘時間的格式化字串，如果無法估算則返回 "unknown"
///
/// # 範例
/// ```text
/// use std::time::Duration;
///
/// let elapsed = Duration::from_secs(60); // 已經過 1 分鐘
/// let remaining = format_remaining_time(elapsed, 100, 1000); // 完成了 100/1000
/// // 輸出類似: "estimated remaining: 9 minutes 0.00 seconds"
/// ```
pub fn format_remaining_time(elapsed: Duration, completed: usize, total: usize) -> String {
    if completed == 0 || completed >= total {
        return "unknown".to_string();
    }

    let elapsed_seconds = elapsed.as_secs_f64();
    let progress_ratio = completed as f64 / total as f64;
    let estimated_total_seconds = elapsed_seconds / progress_ratio;
    let remaining_seconds = estimated_total_seconds - elapsed_seconds;

    if remaining_seconds <= 0.0 {
        return "completing soon".to_string();
    }

    let remaining_duration = Duration::from_secs_f64(remaining_seconds);
    format!(
        "estimated remaining: {}",
        format_duration(remaining_duration)
    )
}

/// 格式化百分比
///
/// # 參數
/// * `completed` - 已完成的數量
/// * `total` - 總數量
/// * `decimal_places` - 小數位數（預設為 1）
///
/// # 返回值
/// 格式化的百分比字串
///
/// # 範例
/// ```text
/// assert_eq!(format_percentage(25, 100, Some(1)), "25.0%");
/// assert_eq!(format_percentage(1, 3, Some(2)), "33.33%");
/// assert_eq!(format_percentage(0, 100, None), "0.0%");
/// ```
#[allow(dead_code)]
pub fn format_percentage(completed: usize, total: usize, decimal_places: Option<usize>) -> String {
    if total == 0 {
        return "0.0%".to_string();
    }

    let percentage = (completed as f64 / total as f64) * 100.0;
    let decimal_places = decimal_places.unwrap_or(1);

    format!("{percentage:.decimal_places$}%")
}

/// 格式化進度條
///
/// # 參數
/// * `completed` - 已完成的數量
/// * `total` - 總數量  
/// * `width` - 進度條寬度（字符數）
///
/// # 返回值
/// 視覺化的進度條字串
///
/// # 範例
/// ```text
/// let bar = format_progress_bar(25, 100, 20);
/// // 輸出: "[#####               ] 25.0%"
/// ```
#[allow(dead_code)]
pub fn format_progress_bar(completed: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return format!("[{}] 0.0%", " ".repeat(width));
    }

    let percentage = completed as f64 / total as f64;
    let filled_width = (percentage * width as f64).round() as usize;
    let empty_width = width.saturating_sub(filled_width);

    format!(
        "[{}{}] {}",
        "#".repeat(filled_width),
        " ".repeat(empty_width),
        format_percentage(completed, total, Some(1))
    )
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::time::Duration;

//     #[test]
//     fn test_format_duration() {
//         let duration = Duration::from_secs(3725); // 1 小時 2 分鐘 5 秒
//         let formatted = format_duration(duration);
//         assert_eq!(formatted, "1 hour 2 minutes 5.00 seconds");

//         let duration = Duration::from_secs(60); // 1 分鐘
//         let formatted = format_duration(duration);
//         assert_eq!(formatted, "1 minute 0.00 seconds");

//         let duration = Duration::from_secs(30); // 30 秒
//         let formatted = format_duration(duration);
//         assert_eq!(formatted, "30.00 seconds");
//     }

//     #[test]
//     fn test_format_number_with_commas() {
//         assert_eq!(format_number_with_commas(1234567), "1,234,567");
//         assert_eq!(format_number_with_commas(1000), "1,000");
//         assert_eq!(format_number_with_commas(123), "123");
//         assert_eq!(format_number_with_commas(0), "0");
//     }

//     #[test]
//     fn test_format_percentage() {
//         assert_eq!(format_percentage(25, 100, Some(1)), "25.0%");
//         assert_eq!(format_percentage(1, 3, Some(2)), "33.33%");
//         assert_eq!(format_percentage(0, 100, None), "0.0%");
//         assert_eq!(format_percentage(0, 0, None), "0.0%"); // 邊界情況
//     }

//     #[test]
//     fn test_format_progress_bar() {
//         let bar = format_progress_bar(25, 100, 20);
//         assert!(bar.contains("25.0%"));
//         assert!(bar.contains("[#####"));

//         let bar = format_progress_bar(0, 100, 10);
//         assert!(bar.contains("0.0%"));
//         assert!(bar.contains("[          ]"));

//         let bar = format_progress_bar(100, 100, 10);
//         assert!(bar.contains("100.0%"));
//         assert!(bar.contains("[##########]"));
//     }

//     #[test]
//     fn test_format_remaining_time() {
//         let elapsed = Duration::from_secs(60); // 已經過 1 分鐘
//         let remaining = format_remaining_time(elapsed, 100, 1000); // 完成了 100/1000
//         assert!(remaining.contains("estimated remaining"));

//         // 測試邊界情況
//         let remaining = format_remaining_time(elapsed, 0, 1000);
//         assert_eq!(remaining, "unknown");

//         let remaining = format_remaining_time(elapsed, 1000, 1000);
//         assert_eq!(remaining, "unknown");
//     }
// }
