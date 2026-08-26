//! CRC32 (IEEE 802.3) checksum implementation.
//!
//! 使用編譯期產生的查找表實作標準 CRC32（多項式 0xEDB88320），
//! 用於逐筆槽位資料的完整性驗證，避免引入外部依賴。

/// 編譯期產生的 CRC32 查找表
const CRC_TABLE: [u32; 256] = build_table();

const fn build_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
            bit += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

/// Streaming CRC32 hasher.
pub struct Crc32 {
    state: u32,
}

impl Crc32 {
    /// Create a hasher with the standard initial state.
    pub fn new() -> Self {
        Self { state: 0xFFFF_FFFF }
    }

    /// Feed bytes into the hasher.
    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            let index = ((self.state ^ byte as u32) & 0xFF) as usize;
            self.state = (self.state >> 8) ^ CRC_TABLE[index];
        }
    }

    /// Consume the hasher and return the final checksum.
    pub fn finalize(self) -> u32 {
        !self.state
    }
}

impl Default for Crc32 {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute the CRC32 of a byte slice in one call.
#[allow(dead_code)]
pub fn crc32(data: &[u8]) -> u32 {
    let mut hasher = Crc32::new();
    hasher.update(data);
    hasher.finalize()
}
