# Release Notes v0.12.0

## Overview

This release redesigns the crate's public API from the ground up. What used to be a single struct that mixed configuration, computation, I/O, analysis, and console output is now seven single-purpose types following modern Rust library conventions: the library never prints, never panics on invalid input, and never deletes files on its own — those responsibilities belong to the caller, and the bundled CLI now implements them through the same public API that external users see. The on-disk file format (`EIGENVALS_V7`) and the data layout are unchanged; only the programmatic surface moves.

The public API is now: `EigenvalueSimulation`, `SimulationData`, `SimulationRecord`, `Statistic`, `Progress`, `JohansenModel`, and `Error`. See the rewritten `LIBRARY_USAGE.md` for the full reference.

## Breaking Changes

### `EigenvalueSimulation`

- Fields (`model`, `dim`, `steps`, `num_runs`) are now private, read back through same-named getters. Struct-literal construction can no longer bypass the validation in `new()`.
- `new()` returns `Result<Self, Error>` (`Error::InvalidConfig`) instead of panicking on out-of-range parameters.
- The data root directory is configurable via the new `with_data_dir()` builder method (default remains `data/` under the working directory).
- `get_file_path(&self, model) -> PathBuf` becomes `data_path(&self) -> PathBuf`: the redundant `model` parameter is gone (the configuration already carries the model).
- `run_simulation()` / `run_simulation_quiet()` become `run() -> Result<(), Error>` and `run_with_progress(impl FnMut(Progress)) -> Result<(), Error>`. The library no longer writes progress or diagnostics to stdout, and no longer panics on failure. The first `Progress { completed, total }` callback reports the resume point right after the file is opened (`completed == total` means the file was already complete); subsequent callbacks arrive after each batch is flushed.
- Running against an incompatible existing file (wrong magic, conflicting parameters, corrupted structure) now returns an error instead of silently deleting and recreating the file. `Error::is_recreatable()` tells the caller whether delete-and-recompute would succeed, and the new public `delete_data()` performs the deletion explicitly. The CLI reimplements the previous warn-then-recreate-once behavior on top of these primitives.
- `read_data()` / `read_all_data()` become `load()` / `load_partial()`, both returning the new `SimulationData` type instead of `Vec<Vec<f64>>` / `Vec<(u32, Vec<f64>)>`. `load()` is strict (any missing record yields `Error::Incomplete`); `load_partial()` returns whatever valid records exist, skipping gaps.
- `verify_file()` is renamed to `verify()`.
- `calculate_trace_percentiles()` / `calculate_maxeig_percentiles()` are removed; percentile analysis moves onto `SimulationData::percentiles(Statistic, &[f64])`, which operates on already-loaded data (one file read instead of one per statistic) and cannot fail. The internal `Aggregator` trait family (`SumAggregator`, `MaxAggregator`, `calculate_percentiles`) is deleted.

### `JohansenModel`

- `to_number()` / `from_number()` are replaced by the idiomatic conversions `impl From<JohansenModel> for u8` and `impl TryFrom<u8> for JohansenModel` (the latter returns `Error::InvalidConfig` outside `0..=4`).
- `all_models()` is replaced by the associated constant `JohansenModel::ALL`.
- `description()` folds into the existing `Display` implementation.
- The four model-property query methods (`has_intercept`, `has_trend`, `intercept_fully_explained_by_cointegration`, `trend_fully_explained_by_cointegration`) are removed; they were metadata used by nothing but their own tests.

### Error handling

- The crate now exposes a single crate-wide error type: `StorageError` (previously `pub(crate)`) becomes `Error`, moves from `src/data_storage/error.rs` to `src/error.rs`, and is re-exported at the crate root.
- Every fallible public API returns `Result<_, Error>`. The previous boundary conversions — flattening structured errors into `io::Error` with `ErrorKind::InvalidData`, and `Box<dyn Error>` from the percentile methods — are gone, so callers can match on error variants instead of parsing message strings.
- The enum is `#[non_exhaustive]` (future variants are non-breaking) and gains an `InvalidConfig` variant for parameter validation. `is_recreatable()` remains public.

## New Features

- `SimulationData`: in-memory result set with `records()`, `is_complete()`, `len()`, `is_empty()`, and `percentiles(Statistic, &[f64])` (linear interpolation; returns one value per requested percentile, NaN when empty).
- `SimulationRecord { seed, eigenvalues }`: records now carry their seed explicitly in both load paths.
- `Statistic::{Trace, MaxEigenvalue}`: the two built-in test statistics; anything else can be computed directly from `records()`.
- `Progress { completed, total }`: progress reporting as data instead of console output.
- `with_data_dir()`: run simulations and read results under any root directory, which also gives tests and embedders clean isolation.

## Refactoring and Improvements

- The binary now consumes the library's public API (`use johansen_null_eigenspectra::...`) instead of re-declaring the source modules a second time. This removes every `#[allow(dead_code)] // library API` annotation, moves the unit-test tree into the library target, and leaves `cli.rs` and `display_utils.rs` as binary-only modules. Presentation logic (progress rendering, statistics tables, the recreate policy) lives in `main.rs`.
- `src/data_storage/parallel_compute.rs`: the compute engine reports through the progress callback, returns errors instead of panicking, and contains no console output; `src/data_storage/writer.rs` likewise drops its `quiet` parameter and all printing.
- `display_utils` sheds its unused helpers (`format_percentage`, `format_progress_bar`).
- Percentile sorting uses `f64::total_cmp` instead of `partial_cmp(...).unwrap()`, removing a latent panic on NaN input.

## Documentation Updates

- `LIBRARY_USAGE.md`: rewritten in full for the new API — type overview, per-method reference, the delete-and-recreate pattern, and an updated end-to-end example.
- `examples/library_usage_example.rs`: rewritten to demonstrate `run_with_progress`, `load`, and `SimulationData::percentiles` with proper `?`-based error handling.
- `DATA_FORMAT.md`: the integrity-check reference now names `verify()`.
- Crate-level documentation added to `src/lib.rs`.

## Testing

- Integration tests now run in per-test temporary directories via `with_data_dir` and no longer write into the project's `data/` directory.
- New coverage: percentile math and statistic aggregation (`simulation_data_test.rs`), invalid-configuration rejection, progress-event sequences, model-number round-trips, custom data roots, and the new incompatible-file contract (`run()` reports a recreatable error and must not delete the file itself).
- 62 tests pass (59 library, 3 binary); `cargo fmt --check` and `cargo clippy --all-targets` are clean. Verified end to end via CLI runs covering the fresh-run, already-complete, resume, and capacity-expansion paths, in both progress and `--quiet` modes.
