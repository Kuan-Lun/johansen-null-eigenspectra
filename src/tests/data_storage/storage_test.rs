//! 儲存層測試：SlotWriter 的寫入、斷點續傳、torn-write 偵測、擴容與完成協議
//!
//! 使用系統暫存目錄，不觸碰專案的 data/ 資料夾。

use crate::data_storage::file_format::{
    FileHeader, STATUS_COMPLETE, STATUS_IN_PROGRESS, crc_path, hash_path,
};
use crate::data_storage::reader::{open_data_file, read_slots};
use crate::data_storage::writer::{OpenResult, SlotWriter, sha256_file};
use std::path::{Path, PathBuf};

fn temp_data_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("johansen_v7_storage_tests");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(format!("{name}.dat"))
}

fn cleanup(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(crc_path(path));
    let _ = std::fs::remove_file(hash_path(path));
}

fn header(num_runs: u64, eigenvalues_per_run: u8) -> FileHeader {
    FileHeader {
        model: 0,
        dim: eigenvalues_per_run,
        steps: 100,
        num_runs,
        eigenvalues_per_run,
        status: STATUS_IN_PROGRESS,
    }
}

/// 產生 seed 專屬的可重現測試數值
fn slot_values(seed: u32, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| seed as f64 + i as f64 * 0.5 + 0.25)
        .collect()
}

#[test]
fn test_write_read_roundtrip_out_of_order() {
    let path = temp_data_path("roundtrip");
    cleanup(&path);
    let path_str = path.to_str().unwrap();

    let OpenResult::InProgress {
        writer,
        remaining_seeds,
        completed_runs,
    } = SlotWriter::open(path_str, header(3, 2), true).unwrap()
    else {
        panic!("expected InProgress for a new file")
    };
    assert_eq!(remaining_seeds, vec![1, 2, 3]);
    assert_eq!(completed_runs, 0);

    // 亂序寫入：slot 位置由 seed 決定，寫入順序無關緊要
    for &seed in &[3u32, 1, 2] {
        writer.write_slot(seed, &slot_values(seed, 2)).unwrap();
    }
    writer.finish().unwrap();

    // 完成協議：status=1、sidecar 刪除、雜湊檔存在
    let data = open_data_file(&path).unwrap();
    assert_eq!(data.header.status, STATUS_COMPLETE);
    assert!(!crc_path(&path).exists());
    assert!(hash_path(&path).exists());

    // 讀取結果天然依 seed 順序
    let records = read_slots(&data.file, &data.header, 3).unwrap();
    for seed in 1..=3u32 {
        assert_eq!(records[(seed - 1) as usize], slot_values(seed, 2));
    }

    // 雜湊檔內容與實際檔案相符
    let content = std::fs::read_to_string(hash_path(&path)).unwrap();
    let recorded_hash = content.split_whitespace().next().unwrap();
    assert_eq!(recorded_hash, sha256_file(&path).unwrap());

    cleanup(&path);
}

#[test]
fn test_resume_detects_missing_and_torn_slots() {
    let path = temp_data_path("resume_torn");
    cleanup(&path);
    let path_str = path.to_str().unwrap();

    // 寫入 seeds 1,2,4（缺 3,5），不執行 finish（模擬中斷）
    {
        let OpenResult::InProgress { writer, .. } =
            SlotWriter::open(path_str, header(5, 1), true).unwrap()
        else {
            panic!("expected InProgress")
        };
        for &seed in &[1u32, 2, 4] {
            writer.write_slot(seed, &slot_values(seed, 1)).unwrap();
        }
        writer.sync().unwrap();
    }

    // 模擬 torn write：翻轉 seed=2 槽位的一個 byte
    {
        use std::io::{Read, Seek, SeekFrom, Write};
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let offset = header(5, 1).slot_offset(2);
        let mut byte = [0u8; 1];
        file.seek(SeekFrom::Start(offset)).unwrap();
        file.read_exact(&mut byte).unwrap();
        file.seek(SeekFrom::Start(offset)).unwrap();
        file.write_all(&[byte[0] ^ 0xFF]).unwrap();
    }

    // 重新打開：seed 2（CRC 不符）與 3,5（全零）都應列為未完成
    let OpenResult::InProgress {
        writer,
        remaining_seeds,
        completed_runs,
    } = SlotWriter::open(path_str, header(5, 1), true).unwrap()
    else {
        panic!("expected InProgress on resume")
    };
    assert_eq!(remaining_seeds, vec![2, 3, 5]);
    assert_eq!(completed_runs, 2);

    // 補齊並完成
    for &seed in &remaining_seeds {
        writer.write_slot(seed, &slot_values(seed, 1)).unwrap();
    }
    writer.finish().unwrap();

    let data = open_data_file(&path).unwrap();
    assert_eq!(data.header.status, STATUS_COMPLETE);
    let records = read_slots(&data.file, &data.header, 5).unwrap();
    for seed in 1..=5u32 {
        assert_eq!(records[(seed - 1) as usize], slot_values(seed, 1));
    }

    cleanup(&path);
}

#[test]
fn test_missing_crc_sidecar_degraded_rebuild() {
    let path = temp_data_path("degraded");
    cleanup(&path);
    let path_str = path.to_str().unwrap();

    // 全部寫入但不 finish，然後刪除 sidecar（模擬人為介入）
    {
        let OpenResult::InProgress { writer, .. } =
            SlotWriter::open(path_str, header(3, 1), true).unwrap()
        else {
            panic!("expected InProgress")
        };
        for seed in 1..=3u32 {
            writer.write_slot(seed, &slot_values(seed, 1)).unwrap();
        }
        writer.sync().unwrap();
    }
    std::fs::remove_file(crc_path(&path)).unwrap();

    // 重開：降級模式將非零槽位視為有效，且因全部完成而自動收尾
    match SlotWriter::open(path_str, header(3, 1), true).unwrap() {
        OpenResult::AlreadyComplete => {}
        _ => panic!("expected AlreadyComplete after degraded rebuild"),
    }
    let data = open_data_file(&path).unwrap();
    assert_eq!(data.header.status, STATUS_COMPLETE);
    assert!(hash_path(&path).exists());
    assert!(!crc_path(&path).exists());

    cleanup(&path);
}

#[test]
fn test_expand_capacity_preserves_existing_records() {
    let path = temp_data_path("expand");
    cleanup(&path);
    let path_str = path.to_str().unwrap();

    // 建立並完成容量 3 的檔案
    {
        let OpenResult::InProgress { writer, .. } =
            SlotWriter::open(path_str, header(3, 2), true).unwrap()
        else {
            panic!("expected InProgress")
        };
        for seed in 1..=3u32 {
            writer.write_slot(seed, &slot_values(seed, 2)).unwrap();
        }
        writer.finish().unwrap();
    }

    // 以較大的 num_runs 重開：應擴容並保留既有資料
    let OpenResult::InProgress {
        writer,
        remaining_seeds,
        completed_runs,
    } = SlotWriter::open(path_str, header(5, 2), true).unwrap()
    else {
        panic!("expected InProgress after expansion")
    };
    assert_eq!(remaining_seeds, vec![4, 5]);
    assert_eq!(completed_runs, 3);

    for &seed in &remaining_seeds {
        writer.write_slot(seed, &slot_values(seed, 2)).unwrap();
    }
    writer.finish().unwrap();

    let data = open_data_file(&path).unwrap();
    assert_eq!(data.header.num_runs, 5);
    assert_eq!(data.header.status, STATUS_COMPLETE);
    let records = read_slots(&data.file, &data.header, 5).unwrap();
    for seed in 1..=5u32 {
        assert_eq!(records[(seed - 1) as usize], slot_values(seed, 2));
    }

    cleanup(&path);
}

#[test]
fn test_param_mismatch_is_recreatable_error() {
    let path = temp_data_path("mismatch");
    cleanup(&path);
    let path_str = path.to_str().unwrap();

    {
        let OpenResult::InProgress { writer, .. } =
            SlotWriter::open(path_str, header(2, 1), true).unwrap()
        else {
            panic!("expected InProgress")
        };
        writer.write_slot(1, &slot_values(1, 1)).unwrap();
        writer.sync().unwrap();
    }

    // steps 不同 → 應回報可重建的參數不相容錯誤
    let mut other = header(2, 1);
    other.steps = 999;
    match SlotWriter::open(path_str, other, true) {
        Err(e) => assert!(
            e.is_recreatable(),
            "param mismatch should be a recreatable error, got: {e}"
        ),
        Ok(_) => panic!("expected a parameter mismatch error"),
    }

    cleanup(&path);
}
