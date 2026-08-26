# Release Notes v0.9.0

## Overview

This release completely redesigns the on-disk storage layer, replacing the append-based `EIGENVALS_V6` format with the fixed-slot `EIGENVALS_V7` format. Seeds are no longer stored in data files: each record lives at a byte offset determined by its seed, enabling lock-free parallel writes, O(1) random access, naturally ordered reads, and zero per-record overhead in completed files. The release also upgrades the Rust toolchain and every dependency to its latest version.

## Breaking Changes

### New V7 Data File Format

- The `EIGENVALS_V6` append format is no longer supported. Existing V6 files are detected via the magic header and automatically removed and recomputed — all data is deterministically recomputable from seeds, so no migration tool is provided.
- Each simulation now produces up to three files:
  - `<name>.dat` — fixed-slot eigenvalue data (permanent)
  - `<name>.dat.crc` — per-record CRC32 sidecar (present only during computation)
  - `<name>.dat.sha256` — whole-file SHA-256 hash in `sha256sum` format (written on completion)
- See `DATA_FORMAT.md` for the full specification.

### API Changes

- `read_data()` now returns `Vec<Vec<f64>>` in seed order (index `i` holds the record for seed `i + 1`), instead of `Vec<(u32, Vec<f64>)>`. Use `read_all_data()` when explicit seeds are needed (e.g., inspecting incomplete files).
- `EigenvalueSimulation::new()` now validates its parameters and panics on out-of-range values: `dim` must be in `1..=254`, `steps` and `num_runs` in `1..=u32::MAX`.

### Reproducibility Across Versions

- The toolchain and dependency upgrades shift eigenvalue results by linear-algebra rounding noise (maximum absolute difference ~6e-14; ~1e-12 relative on significant values). RNG streams were verified bitwise-identical across versions; the differences come solely from floating-point operation ordering in the matrix backends.
- Results are statistically unchanged, but files generated with earlier releases are not bitwise reproducible under v0.9.0. Long-term archived datasets should be regenerated to align with the new lockfile.

## New Features

- **Fixed-slot storage with implicit seeds**: `slot_offset(seed) = header + (seed − 1) × N × 8`; files are pre-allocated with `set_len` and benefit from sparse-file support.
- **Torn-write detection**: during computation, each record is protected by a CRC32 computed over `seed ‖ payload`, which also verifies that records occupy their correct slots. Resume scans recompute exactly the invalid seeds.
- **Whole-file integrity verification**: completed files carry a SHA-256 hash file compatible with `shasum -a 256 -c`; the new `verify_file()` API performs the check programmatically.
- **In-place capacity expansion**: reopening a file with a larger `num_runs` extends it while preserving all existing records.
- **New public APIs**: `verify_file()`, `remove_data_files()`, and `JohansenModel::eigenvalues_per_run()`.

## Refactoring and Improvements

- Workers now write results directly into their slots via positioned writes (`pwrite`); the mpsc channel and dedicated writer thread were removed, along with the ULEB128 codec and scan-based reading.
- A structured `StorageError` enum replaces string-matched error classification.
- Completed files are read without per-record validation (the completion protocol guarantees durability), keeping the fast read path.
- CLI argument parsing flattened a vestigial `Option<Option<T>>` nesting.
- Resume progress checks now stream through files instead of loading all records into memory.

## Dependency and Toolchain Upgrades

| Component | Old | New |
|-----------|-----|-----|
| rustc | 1.90.0 | 1.98.0 |
| rand | 0.9.1 | 0.10.2 |
| rand_distr | 0.5.1 | 0.6.0 |
| rand_xoshiro | 0.7.0 | 0.8.1 |
| rayon | 1.10.0 | 1.12.0 |
| sha2 | (new) 0.10 | 0.11.0 |
| nalgebra (git) | 0.33.2 | 0.35.0 |
| nalgebra-lapack (git) | 0.25.0 | 0.28.0 |

- `rand 0.10` moved `Rng::random` to the `RngExt` trait; call sites updated.
- `nalgebra-lapack` now uses its `lapack-custom` feature, the upstream-supported way to link an external LAPACK via `.cargo/config.toml` (Accelerate on macOS, system liblapack on Linux). The rationale for staying on the git dependency is documented in `Cargo.toml`.

## Documentation Updates

- `DATA_FORMAT.md` rewritten for the V7 specification: layout, CRC sidecar, hash file, and lifecycle protocols (creation, resume, completion, expansion).
- `LIBRARY_USAGE.md` updated for the new `read_data` return type and `verify_file()` API.
- `AGENTS.md` gained a build-environment section (cargo must run from the project root for LAPACK linking).
- `.agents/code-generation.md` now distinguishes statistical from bitwise reproducibility and documents the dataset-regeneration policy after upgrades.

## Testing

- New `format_test.rs`: header codec round-trips, CRC32 test vectors, slot-offset arithmetic, and the record-validity rules.
- New `storage_test.rs`: out-of-order writes, resume after simulated interruption, torn-write detection via bit corruption, degraded rebuild when the CRC sidecar is missing, capacity expansion, and parameter-mismatch handling.
- Integration tests updated for the V7 semantics; "removing" a record is now modeled by zeroing its slot.
- All 59 tests pass; `cargo clippy --all-targets` is warning-free on Rust 1.98.

## Implementation Details

### Files Added

- `src/data_storage/crc32.rs` — table-based CRC32 (IEEE), no external dependency
- `src/data_storage/error.rs` — structured storage error types
- `src/tests/data_storage/format_test.rs`, `src/tests/data_storage/storage_test.rs`

### Files Removed

- `src/data_storage/uleb128.rs`, `src/data_storage/thread_manager.rs`, `src/data_storage/progress.rs`
- `src/tests/data_storage/append_writer_test.rs`, `uleb128_test.rs`, `uleb128_unit_test.rs`

### Files Rewritten

- `src/data_storage/file_format.rs`, `writer.rs`, `reader.rs`, `parallel_compute.rs`, `simulation.rs`
