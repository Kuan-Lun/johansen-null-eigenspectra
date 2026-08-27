# Johansen Null Eigenspectra Data File Format Specification

## Overview

This project uses a custom binary file format `EIGENVALS_V7` to efficiently store eigenvalue simulation results from Johansen cointegration tests. The format uses a **fixed-slot layout**: seeds are never stored in the file. Instead, the record for seed `s` lives at a fixed byte offset determined by `s`, so seeds are implicit in record positions. This enables lock-free parallel positioned writes, O(1) random access reads, and naturally ordered data.

Each simulation produces up to three files:

| File                | Lifetime                | Purpose                                                       |
|---------------------|-------------------------|---------------------------------------------------------------|
| `<name>.dat`        | permanent               | Fixed-slot eigenvalue data                                    |
| `<name>.dat.crc`    | during computation only | Per-record CRC32 sidecar for resume and torn-write detection  |
| `<name>.dat.sha256` | after completion        | Whole-file SHA-256 hash for archival integrity                |

## Data File (`.dat`)

```text
[Header: 32 bytes] + [Slots: num_runs × slot_size bytes]
```

### Header — 32 bytes

| Offset | Size | Type   | Description                           | Example Value                         |
|--------|------|--------|---------------------------------------|---------------------------------------|
| 0      | 12   | ASCII  | Magic header "EIGENVALS_V7"           | `45 49 47 45 4E 56 41 4C 53 5F 56 37` |
| 12     | 1    | u8     | Johansen model number                 | `00` (model 0)                        |
| 13     | 1    | u8     | Time series dimension                 | `02` (2 dimensions)                   |
| 14     | 4    | u32 LE | Number of simulation steps            | `32 00 00 00` (50 steps)              |
| 18     | 8    | u64 LE | Number of runs (file capacity)        | `C8 00 00 00 00 00 00 00` (200 runs)  |
| 26     | 1    | u8     | Eigenvalues per record (N)            | `02` (2 eigenvalues)                  |
| 27     | 1    | u8     | Status: 0 = in progress, 1 = complete | `01`                                  |
| 28     | 4    | —      | Reserved (zero)                       | `00 00 00 00`                         |

Notes:

- `dim` is validated to `1..=254` so that `dim + 1` eigenvalues always fit in a byte.
- The eigenvalue count N equals `dim` or `dim + 1` depending on the model (models 1 and 3 use `dim + 1`). It is stored so the file is self-describing and is cross-checked against the model rule on open.
- `num_runs` is capped at `u32::MAX` because seeds are 32-bit.

#### Model Number Mapping

| Number | Model Description                                                           |
|--------|-----------------------------------------------------------------------------|
| 0      | No intercept, no trend                                                      |
| 1      | Intercept, no trend, intercept in cointegration                             |
| 2      | Intercept, no trend, intercept not fully explained by cointegration         |
| 3      | Intercept, trend, trend in cointegration                                    |
| 4      | Intercept, trend, intercept and trend not fully explained by cointegration  |

### Slots

Each slot holds one Monte Carlo record as N consecutive f64 values (little-endian). There is no per-record seed, count, or terminator — the layout is fully determined by the header:

```text
slot_size          = N × 8
slot_offset(seed)  = 32 + (seed − 1) × slot_size      (seeds are 1-based)
file_size          = 32 + num_runs × slot_size
```

An **empty slot is all zeros**. Johansen eigenvalues are almost surely positive, so a zero payload never represents a real record. Files are pre-allocated to full size with `set_len` at creation; on file systems with sparse file support the unwritten region occupies no physical disk space.

The file size is fixed at creation and never changes during computation; workers fill slots in place with positioned writes (`pwrite`). Readers require `actual_size ≥ file_size` implied by the header (a larger file can only result from an interrupted capacity expansion; the excess tail is ignored and trimmed on the next write open).

## CRC Sidecar (`.crc`)

Present only while a simulation is in progress (status = 0). Provides per-record validation for resume: it detects both unfinished slots and torn writes (records only partially persisted before a crash or power loss).

```text
[Header: 16 bytes] + [num_runs × 4 bytes]
```

| Offset | Size | Type   | Description                                             |
|--------|------|--------|---------------------------------------------------------|
| 0      | 8    | ASCII  | Magic "EIGCRC_7"                                        |
| 8      | 8    | u64 LE | num_runs (must match the data file)                     |
| 16     | 4×n  | u32 LE | CRC per slot: `crc_offset(seed) = 16 + (seed − 1) × 4`  |

The checksum is standard CRC32 (IEEE 802.3, polynomial `0xEDB88320`) computed over `seed_le_bytes ‖ payload_bytes`. Including the seed binds each record to its slot, so a record accidentally written to the wrong position also fails validation.

**Validity rule**: a slot is valid ⟺ `stored_crc == CRC32(seed ‖ payload)` **and** the payload is not all zeros.

If the sidecar is missing while status = 0 (e.g. deleted manually), the reader falls back to a degraded rule — "non-zero payload = valid" — with a warning, and the sidecar is rebuilt from the existing data. Torn-write detection is unavailable for pre-existing records in that case.

## Hash File (`.sha256`)

Written at completion. Contains the SHA-256 of the final data file in standard `sha256sum` format:

```text
<64 hex digits>  <basename of the data file>
```

It can be verified with the library API (`EigenvalueSimulation::verify_file()`) or externally:

```sh
shasum -a 256 -c eigenvalues_steps50_model0_dim2.dat.sha256
```

Verification is on demand only; ordinary reads do not pay the whole-file hashing cost.

## Lifecycle

### Creation

1. Write the data file header with status = 0, `set_len` to full size (sparse).
2. Write the sidecar header, `set_len` to full size.

### Writing

Workers compute records in parallel and write each record directly into its slot, followed by its CRC entry. Slots do not overlap, so no locking is required. Both files are `fsync`ed after every batch (10,000 records); a crash loses at most the last batch, which the resume scan detects and recomputes.

### Completion protocol

1. `fsync` data and sidecar.
2. Write status = 1 at header offset 27; `fsync`.
3. Compute SHA-256 of the final data file and write the `.sha256` file.
4. Delete the `.crc` sidecar.

Step ordering guarantees that status = 1 implies all records are durably on disk, so completed files are read with no per-record validation.

### Resume (status = 0)

1. Open and cross-check both headers (magic, model, dim, steps, N, num_runs).
2. Sequentially scan all slots, applying the validity rule.
3. Recompute exactly the invalid seeds; every record is deterministic given its seed, so the seed itself acts as the redo log.

### Capacity expansion

Opening an existing file with a larger `num_runs` extends it in place: both files are `set_len` to the new size, headers are updated, status returns to 0, and the stale `.sha256` is deleted. Existing records are untouched. For a previously completed file (whose sidecar was deleted), the sidecar is rebuilt from the stored records. Opening with a smaller `num_runs` simply reads a prefix of the slots.

### Incompatible files

A file whose magic, model, dim, steps, or eigenvalue count conflicts with the requested configuration (or whose structure is corrupt) is deleted and recreated — all data is recomputable from seeds.

## Worked Example

`data/steps50/model0/eigenvalues_steps50_model0_dim2.dat` with 200 runs, N = 2:

```text
Header:
45 49 47 45 4E 56 41 4C 53 5F 56 37   "EIGENVALS_V7"
00                                     model 0
02                                     dim 2
32 00 00 00                            steps = 50
C8 00 00 00 00 00 00 00                num_runs = 200
02                                     N = 2 eigenvalues per record
01                                     status = complete
00 00 00 00                            reserved

Slots:
offset 32:   record for seed 1   (2 × f64 LE)
offset 48:   record for seed 2
...
offset 3216: record for seed 200
file size = 32 + 200 × 16 = 3,232 bytes
```

## File Size

```text
data file  = 32 + num_runs × N × 8            (permanent)
sidecar    = 16 + num_runs × 4                (during computation only)
hash file  ≈ 100 bytes                        (after completion)
```

For 10,000,000 runs at dim 1 the permanent file is exactly 80,000,032 bytes with zero per-record overhead — no seeds, counts, or markers are stored.

## Version Compatibility

- **Current Version**: V7 (magic header `EIGENVALS_V7`)
- **Legacy Files**: V6 and older append-format files are not supported; they are detected via the magic header mismatch and recreated.
- **Format Evolution**: the version number in the magic header supports future extensions.

## Related Code

Main implementation files:

- `src/data_storage/file_format.rs`: format constants, header codec, offsets, validity rule
- `src/data_storage/writer.rs`: positioned slot writer, resume scan, expansion, completion protocol
- `src/data_storage/reader.rs`: fast read and validating scan paths
- `src/data_storage/crc32.rs`: CRC32 implementation
- `src/tests/data_storage/`: format and storage-layer test cases
