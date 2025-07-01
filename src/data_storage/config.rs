/// 配置常數
const DEFAULT_BATCH_SIZE: usize = 1e4 as usize; // 預設批次大小
pub const BATCH_SIZE: usize = DEFAULT_BATCH_SIZE;
pub const PROGRESS_REPORT_INTERVAL: usize = DEFAULT_BATCH_SIZE;
pub const FLUSH_INTERVAL: usize = DEFAULT_BATCH_SIZE;

/// write buffer capacity in bytes for AppendOnlyWriter
pub const WRITE_BUFFER_CAPACITY: usize = 2 * 1024 * 1024; // 2 MiB

/// 讀取緩衝區配置
pub const MIN_READ_BUFFER_SIZE: usize = 64 * 1024; // 64 KB - 最小讀取緩衝區
pub const MAX_READ_BUFFER_SIZE: usize = 16 * 1024 * 1024; // 16 MiB - 最大讀取緩衝區
