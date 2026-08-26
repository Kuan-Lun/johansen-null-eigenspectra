/// 自適應批次配置：每批計算完成後同步落盤並回報進度，
/// 批次大小依實測耗時向目標批次時間收斂。
///
/// 目標批次時間（秒）：控制 checkpoint 間隔，斷電最多損失約此時長的計算量
pub const TARGET_BATCH_SECS: f64 = 10.0;

/// 批次大小下限 = 執行緒數 × 此值（攤平 sync 成本並餵飽核心；與上限衝突時上限優先）
pub const BATCH_FLOOR_PER_THREAD: usize = 100;

/// 批次大小上限 = 總模擬次數 / 此值（保證整輪至少約此數量的 checkpoint 與進度回報）
pub const CHECKPOINT_DIVISOR: usize = 10;

/// 批次調整的平滑係數：每批向目標修正的比例，避免因單批耗時抖動而震盪
pub const BATCH_SMOOTHING: f64 = 0.5;

/// 讀取緩衝區配置
pub const MIN_READ_BUFFER_SIZE: usize = 64 * 1024; // 64 KB - 最小讀取緩衝區
pub const MAX_READ_BUFFER_SIZE: usize = 16 * 1024 * 1024; // 16 MiB - 最大讀取緩衝區
