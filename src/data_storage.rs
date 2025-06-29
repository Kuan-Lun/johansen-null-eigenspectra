use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

/// 將特徵值數據以二進制格式儲存
/// 這種格式儲存空間小，讀取速度快，但不易讀
pub fn write_eigenvalues_binary<P: AsRef<Path>>(data: &[Vec<f64>], path: P) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // 寫入數據的維度信息
    let num_runs = data.len();
    writer.write_all(&(num_runs as u64).to_le_bytes())?;

    if num_runs > 0 {
        let eigenvalues_per_run = data[0].len();
        writer.write_all(&(eigenvalues_per_run as u64).to_le_bytes())?;

        // 寫入所有特徵值
        for eigenvalues in data {
            for &val in eigenvalues {
                writer.write_all(&val.to_le_bytes())?;
            }
        }
    }

    Ok(())
}

/// 從二進制格式讀取特徵值數據
pub fn read_eigenvalues_binary<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<Vec<f64>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut buffer = [0u8; 8];

    // 讀取維度信息
    reader.read_exact(&mut buffer)?;
    let num_runs = u64::from_le_bytes(buffer) as usize;

    if num_runs == 0 {
        return Ok(Vec::new());
    }

    reader.read_exact(&mut buffer)?;
    let eigenvalues_per_run = u64::from_le_bytes(buffer) as usize;

    let mut data = Vec::with_capacity(num_runs);

    // 讀取所有特徵值
    for _ in 0..num_runs {
        let mut eigenvalues = Vec::with_capacity(eigenvalues_per_run);
        for _ in 0..eigenvalues_per_run {
            reader.read_exact(&mut buffer)?;
            eigenvalues.push(f64::from_le_bytes(buffer));
        }
        data.push(eigenvalues);
    }

    Ok(data)
}

/// 將特徵值數據以CSV格式儲存
/// 這種格式易讀，可以用Excel等軟件打開，但檔案較大
pub fn write_eigenvalues_csv<P: AsRef<Path>>(data: &[Vec<f64>], path: P) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // 寫入標題行
    if !data.is_empty() {
        let header: Vec<String> = (0..data[0].len())
            .map(|i| format!("eigenvalue_{}", i + 1))
            .collect();
        writeln!(writer, "{}", header.join(","))?;

        // 寫入數據
        for eigenvalues in data {
            let row: Vec<String> = eigenvalues
                .iter()
                .map(|&val| format!("{:.12}", val))
                .collect();
            writeln!(writer, "{}", row.join(","))?;
        }
    }

    Ok(())
}

/// 從CSV格式讀取特徵值數據
pub fn read_eigenvalues_csv<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<Vec<f64>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // 跳過標題行
    lines.next();

    let mut data = Vec::new();

    for line in lines {
        let line = line?;
        let eigenvalues: Result<Vec<f64>, _> = line.split(',').map(|s| s.trim().parse()).collect();

        match eigenvalues {
            Ok(vals) => data.push(vals),
            Err(_) => continue, // 跳過無法解析的行
        }
    }

    Ok(data)
}

/// 將單次運算的特徵值儲存（簡化版本）
pub fn write_single_eigenvalues<P: AsRef<Path>>(
    eigenvalues: &[f64],
    path: P,
) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for &val in eigenvalues {
        writer.write_all(&val.to_le_bytes())?;
    }

    Ok(())
}

/// 讀取單次運算的特徵值
pub fn read_single_eigenvalues<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<f64>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut eigenvalues = Vec::new();
    let mut buffer = [0u8; 8];

    while reader.read_exact(&mut buffer).is_ok() {
        eigenvalues.push(f64::from_le_bytes(buffer));
    }

    Ok(eigenvalues)
}

/// 將特徵值數據以CSV格式儲存（包含seed）
/// 第一欄是seed，後續欄位是特徵值
pub fn write_eigenvalues_csv_with_seed<P: AsRef<Path>>(
    data: &[(u64, Vec<f64>)],
    path: P,
) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // 寫入標題行
    if !data.is_empty() {
        let mut header = vec!["seed".to_string()];
        let eigenvalue_headers: Vec<String> = (0..data[0].1.len())
            .map(|i| format!("eigenvalue_{}", i + 1))
            .collect();
        header.extend(eigenvalue_headers);
        writeln!(writer, "{}", header.join(","))?;

        // 寫入數據
        for (seed, eigenvalues) in data {
            let mut row = vec![seed.to_string()];
            let eigenvalue_strs: Vec<String> = eigenvalues
                .iter()
                .map(|&val| format!("{:.12}", val))
                .collect();
            row.extend(eigenvalue_strs);
            writeln!(writer, "{}", row.join(","))?;
        }
    }

    Ok(())
}

/// 從CSV格式讀取特徵值數據（包含seed）
pub fn read_eigenvalues_csv_with_seed<P: AsRef<Path>>(
    path: P,
) -> std::io::Result<Vec<(u64, Vec<f64>)>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // 跳過標題行
    lines.next();

    let mut data = Vec::new();

    for line in lines {
        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() < 2 {
            continue; // 跳過格式不正確的行
        }

        // 解析seed
        let seed = match parts[0].trim().parse::<u64>() {
            Ok(s) => s,
            Err(_) => continue,
        };

        // 解析特徵值
        let eigenvalues: Result<Vec<f64>, _> =
            parts[1..].iter().map(|s| s.trim().parse()).collect();

        match eigenvalues {
            Ok(vals) => data.push((seed, vals)),
            Err(_) => continue, // 跳過無法解析的行
        }
    }

    Ok(data)
}

/// 將特徵值數據以二進制格式儲存（包含seed）
pub fn write_eigenvalues_binary_with_seed<P: AsRef<Path>>(
    data: &[(u64, Vec<f64>)],
    path: P,
) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // 寫入數據的維度信息
    let num_runs = data.len();
    writer.write_all(&(num_runs as u64).to_le_bytes())?;

    if num_runs > 0 {
        let eigenvalues_per_run = data[0].1.len();
        writer.write_all(&(eigenvalues_per_run as u64).to_le_bytes())?;

        // 寫入所有數據（seed + 特徵值）
        for (seed, eigenvalues) in data {
            writer.write_all(&seed.to_le_bytes())?;
            for &val in eigenvalues {
                writer.write_all(&val.to_le_bytes())?;
            }
        }
    }

    Ok(())
}

/// 從二進制格式讀取特徵值數據（包含seed）
pub fn read_eigenvalues_binary_with_seed<P: AsRef<Path>>(
    path: P,
) -> std::io::Result<Vec<(u64, Vec<f64>)>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut buffer = [0u8; 8];

    // 讀取維度信息
    reader.read_exact(&mut buffer)?;
    let num_runs = u64::from_le_bytes(buffer) as usize;

    if num_runs == 0 {
        return Ok(Vec::new());
    }

    reader.read_exact(&mut buffer)?;
    let eigenvalues_per_run = u64::from_le_bytes(buffer) as usize;

    let mut data = Vec::with_capacity(num_runs);

    // 讀取所有數據
    for _ in 0..num_runs {
        // 讀取seed
        reader.read_exact(&mut buffer)?;
        let seed = u64::from_le_bytes(buffer);

        // 讀取特徵值
        let mut eigenvalues = Vec::with_capacity(eigenvalues_per_run);
        for _ in 0..eigenvalues_per_run {
            reader.read_exact(&mut buffer)?;
            eigenvalues.push(f64::from_le_bytes(buffer));
        }
        data.push((seed, eigenvalues));
    }

    Ok(data)
}
