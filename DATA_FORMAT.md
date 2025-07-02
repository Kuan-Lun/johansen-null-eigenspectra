# Johansen Null Eigenspectra Data File Format Specification

## Overview

This project uses a custom binary file format `EIGENVALS_V5` to efficiently store eigenvalue simulation results from Johansen cointegration tests. The format is designed for high-performance writing and reading of large-scale simulation data, with support for resume functionality and data integrity verification.

## File Structure

```text
[File Header: 18 bytes] + [Data Records: Variable Length] + [EOF Marker and Metadata: 17 bytes]
```

## Detailed Format Specification

### 1. File Header - 18 bytes

| Offset | Size | Type  | Description | Example Value |
|--------|------|-------|-------------|---------------|
| 0      | 12   | ASCII | Magic header "EIGENVALS_V5" | `45 49 47 45 4E 56 41 4C 53 5F 56 35` |
| 12     | 1    | u8    | Johansen model number | `00` (model 0) |
| 13     | 1    | u8    | Time series dimension | `01` (1 dimension) |
| 14     | 4    | u32   | Number of simulation steps (little-endian) | `0A 00 00 00` (10 steps) |

#### Magic Header Description

- `EIGENVALS_V5`: Indicates file format version 5
- Used for file type identification and format error prevention

#### Model Number Mapping

| Number | Model Description |
|--------|-------------------|
| 0      | No intercept, no trend |
| 1      | Intercept, no trend, intercept in cointegration |
| 2      | Intercept, trend, trend in cointegration |
| ...    | (other models) |

### 2. Data Records Section - Variable Length

Each data record represents one Monte Carlo simulation result:

| Offset | Size | Type | Description |
|--------|------|------|-------------|
| 0      | 4    | u32  | Random seed (little-endian) |
| 4      | 1    | u8   | Number of eigenvalues |
| 5      | 8×N  | f64  | N eigenvalues (8 bytes each, little-endian) |

**Record size calculation**: `5 + 8 × number_of_eigenvalues` bytes

#### Eigenvalue Count Notes

- Usually equals dimension or dimension+1 (depending on model)
- Limited to 0-255 range (u8)
- Must be consistent across all records in the file

### 3. EOF Marker and Metadata - 17 bytes

| Offset | Size | Type  | Description | Example Value |
|--------|------|-------|-------------|---------------|
| 0      | 8    | ASCII | EOF marker "EOF_MARK" | `45 4F 46 5F 4D 41 52 4B` |
| 8      | 8    | u64   | Total record count (little-endian) | `80 96 98 00 00 00 00 00` (10,000,000) |
| 16     | 1    | u8    | Eigenvalues per record | `01` (1 eigenvalue) |

## File Size Calculation

```rust
file_size = header_size + (record_size × record_count) + metadata_size
          = 18 + (5 + 8 × eigenvalues_count) × record_count + 17
```

### Example Calculation

For a 1-dimension, 10,000,000 simulation file:

```text
file_size = 18 + (5 + 8×1) × 10,000,000 + 17
          = 18 + 13 × 10,000,000 + 17  
          = 130,000,035 bytes ≈ 124 MB
```

## Real-World Examples

### File Header Example

```hex
45 49 47 45 4E 56 41 4C 53 5F 56 35 00 01 0A 00 00 00
```

Parsed:

- `45 49 47 45 4E 56 41 4C 53 5F 56 35`: "EIGENVALS_V5"
- `00`: Model 0
- `01`: 1 dimension
- `0A 00 00 00`: 10 steps

### Data Record Example

```hex
01 00 00 00 01 3F F0 00 00 00 00 00 00
```

Parsed:

- `01 00 00 00`: Seed = 1
- `01`: 1 eigenvalue
- `3F F0 00 00 00 00 00 00`: Eigenvalue = 1.0

### EOF Marker Example

```hex
45 4F 46 5F 4D 41 52 4B 80 96 98 00 00 00 00 00 01
```

Parsed:

- `45 4F 46 5F 4D 41 52 4B`: "EOF_MARK"
- `80 96 98 00 00 00 00 00`: 10,000,000 records
- `01`: 1 eigenvalue per record

## File Operation Features

### 1. Append-Only Writing Support

- True append-only writing avoids rewriting entire files
- Resume functionality by removing EOF marker
- Buffered writing for performance

### 2. Data Integrity Checks

- Magic header verifies file format
- Parameter matching validation (model, dimension, steps)
- EOF marker ensures data completeness

### 3. Error Recovery

- Supports reading incomplete files without EOF markers
- Scan-based reading for corrupted files
- Automatic detection and handling of incompatible formats

### 4. Performance Optimizations

- Dynamic read buffer sizing based on file size
- Periodic write buffer flushing
- Large file support (>16MB buffers)

## Reading Strategies

### Fast Reading Mode

When file has complete EOF marker:

1. Read metadata from end to get total record count
2. Jump directly to data section start
3. Read all records sequentially with known structure

### Scan Reading Mode

When file lacks EOF marker:

1. Scan records from data section start
2. Detect EOF marker or zero-filled regions
3. Handle incomplete records gracefully

## Version Compatibility

- **Current Version**: V5
- **Backward Compatibility**: Detects old format versions and prompts for upgrade
- **Format Evolution**: Version number in magic header supports future extensions

## Use Cases

1. **Large-scale Monte Carlo Simulations**: Efficiently store millions of simulation results
2. **Distributed Computing**: Support parallel writing to different files for later merging
3. **Long-running Tasks**: Resume functionality supports task interruption and recovery
4. **Data Analysis**: Fast reading for statistical analysis and visualization

## Related Code

Main implementation files:

- `src/data_storage/append_writer.rs`: Core file I/O logic
- `src/data_storage/config.rs`: Buffer and performance configuration
- `tests/data_storage/`: Format validation and test cases
