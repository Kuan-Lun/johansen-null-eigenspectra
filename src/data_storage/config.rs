/// 並行計算的批次大小：每批計算完成後同步落盤並回報進度
pub const BATCH_SIZE: usize = 1e4 as usize;

/// 讀取緩衝區配置
pub const MIN_READ_BUFFER_SIZE: usize = 64 * 1024; // 64 KB - 最小讀取緩衝區
pub const MAX_READ_BUFFER_SIZE: usize = 16 * 1024 * 1024; // 16 MiB - 最大讀取緩衝區
