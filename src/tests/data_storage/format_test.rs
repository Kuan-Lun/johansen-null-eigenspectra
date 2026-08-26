//! V7 檔案格式單元測試：header 編解碼、CRC 與槽位有效性規則

use crate::data_storage::crc32::crc32;
use crate::data_storage::error::StorageError;
use crate::data_storage::file_format::{
    FileHeader, HEADER_SIZE, STATUS_COMPLETE, STATUS_IN_PROGRESS, record_crc, slot_is_valid,
};

fn sample_header() -> FileHeader {
    FileHeader {
        model: 2,
        dim: 3,
        steps: 1000,
        num_runs: 500,
        eigenvalues_per_run: 3,
        status: STATUS_IN_PROGRESS,
    }
}

#[test]
fn test_crc32_known_vector() {
    // CRC32 (IEEE) 的標準測試向量
    assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    assert_eq!(crc32(b""), 0);
}

#[test]
fn test_header_roundtrip() {
    let header = sample_header();
    let encoded = header.encode();
    assert_eq!(encoded.len() as u64, HEADER_SIZE);
    let decoded = FileHeader::decode(&encoded).unwrap();
    assert_eq!(decoded, header);

    // status 欄位也要能往返
    let mut complete = header;
    complete.status = STATUS_COMPLETE;
    assert_eq!(FileHeader::decode(&complete.encode()).unwrap(), complete);
}

#[test]
fn test_header_decode_rejects_bad_magic() {
    let mut encoded = sample_header().encode();
    encoded[0] ^= 0xFF;
    assert!(matches!(
        FileHeader::decode(&encoded),
        Err(StorageError::MagicMismatch)
    ));
}

#[test]
fn test_header_decode_rejects_invalid_fields() {
    // eigenvalues_per_run 為零
    let mut encoded = sample_header().encode();
    encoded[26] = 0;
    assert!(matches!(
        FileHeader::decode(&encoded),
        Err(StorageError::Corrupted(_))
    ));

    // 未知的 status byte
    let mut encoded = sample_header().encode();
    encoded[27] = 9;
    assert!(matches!(
        FileHeader::decode(&encoded),
        Err(StorageError::Corrupted(_))
    ));
}

#[test]
fn test_slot_offsets() {
    let header = sample_header();
    // slot 大小 = N × 8；seed 1 緊接在 header 之後
    assert_eq!(header.slot_size(), 24);
    assert_eq!(header.slot_offset(1), HEADER_SIZE);
    assert_eq!(header.slot_offset(10), HEADER_SIZE + 9 * 24);
    assert_eq!(
        header.data_file_size(),
        HEADER_SIZE + header.num_runs * header.slot_size()
    );
}

#[test]
fn test_record_crc_binds_seed_to_slot() {
    // 相同 payload、不同 seed 必須產生不同的 CRC（驗證資料寫在正確槽位）
    let payload = 1.5f64.to_le_bytes();
    assert_ne!(record_crc(1, &payload), record_crc(2, &payload));
}

#[test]
fn test_slot_validity_rules() {
    let payload = 1.5f64.to_le_bytes();
    let crc = record_crc(7, &payload);

    // CRC 相符且非全零 → 有效
    assert!(slot_is_valid(7, &payload, Some(crc)));
    // CRC 不符（torn write）→ 無效
    assert!(!slot_is_valid(7, &payload, Some(crc ^ 1)));
    // 寫錯槽位（seed 不同）→ 無效
    assert!(!slot_is_valid(8, &payload, Some(crc)));
    // 全零 payload（空槽位）→ 無效，即使 CRC 欄位恰好相符
    let zeros = [0u8; 8];
    assert!(!slot_is_valid(7, &zeros, Some(record_crc(7, &zeros))));
    // 降級模式（無 CRC sidecar）：非全零即有效
    assert!(slot_is_valid(7, &payload, None));
    assert!(!slot_is_valid(7, &zeros, None));
}

#[test]
fn test_check_compatible() {
    let header = sample_header();
    assert!(header.check_compatible(&header).is_ok());

    // num_runs 與 status 不參與相容性檢查（容量與進度另行處理）
    let mut other = header;
    other.num_runs = 999;
    other.status = STATUS_COMPLETE;
    assert!(header.check_compatible(&other).is_ok());

    let mut mismatched = header;
    mismatched.steps = 2000;
    assert!(matches!(
        header.check_compatible(&mismatched),
        Err(StorageError::ParamMismatch { field: "steps", .. })
    ));
}
