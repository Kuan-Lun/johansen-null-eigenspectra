/// 配置常數
const DEFAULT_BATCH_SIZE: usize = 1e4 as usize; // 預設批次大小
pub const BATCH_SIZE: usize = DEFAULT_BATCH_SIZE;
pub const PROGRESS_REPORT_INTERVAL: usize = DEFAULT_BATCH_SIZE;
pub const FLUSH_INTERVAL: usize = DEFAULT_BATCH_SIZE;

/// write buffer capacity in bytes for AppendOnlyWriter
pub const WRITE_BUFFER_CAPACITY: usize = 2 * 1024 * 1024; // 2 MiB
