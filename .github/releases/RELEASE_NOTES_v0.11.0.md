# Release Notes v0.11.0

## Overview

This release reorganizes how simulation results are stored on disk. Data files previously lived in a single flat `data/` directory whose filenames carried all the parameters (`data/eigenvalues_model0_dim2_steps50.dat`). They now live in a directory hierarchy grouped by steps and then by model, with the filename parameters reordered to match: `data/steps50/model0/eigenvalues_steps50_model0_dim2.dat`. Grouping by steps first reflects how the data is consumed — users pick a steps value and then select among the models under it, and results from different models are not comparable with each other. The hierarchy also makes it practical to archive, sync, or delete one steps campaign or one model at a time. The on-disk file format itself (`EIGENVALS_V7`) is unchanged.

## Breaking Changes

### New data file layout

- Simulation results are written to `data/steps{steps}/model{model}/eigenvalues_steps{steps}_model{model}_dim{dim}.dat` (relative to the working directory) instead of `data/eigenvalues_model{model}_dim{dim}_steps{steps}.dat`. The `.crc` and `.sha256` sidecar files live next to the data file as before.
- There is no migration or fallback for files stored under the old flat layout: existing files are simply not found, and simulations restart from scratch. Regenerate the data or move and rename old files manually if you need to keep them.

### Library API changes

- `EigenvalueSimulation::get_filename(&self, model) -> String` is renamed to `get_file_path(&self, model) -> PathBuf`. The method is now a pure path computation: it no longer creates the `data/` directory as a side effect, so read-only operations leave no empty directories behind. Directories are created by the writer just before a new data file is created.
- `EigenvalueSimulation::remove_data_files` is no longer part of the public API (now `pub(crate)`). It was an internal recovery and test-cleanup utility with no valid external use case.

## Refactoring and Improvements

- `src/data_storage/simulation.rs`: `get_file_path` builds the nested path; internal read/verify/remove operations use `PathBuf` throughout. `remove_data_files` deletes the data file and its sidecars, then removes the emptied `model{model}/` and `steps{steps}/` directories from the inside out (`remove_dir` fails on non-empty directories, so data belonging to other models or steps values is never touched).
- `src/data_storage/writer.rs`: `SlotWriter::open` accepts `impl AsRef<Path>` instead of `&str`, matching the reader's convention; `create_new` creates the parent directories before creating the data file.
- `src/data_storage/parallel_compute.rs`: uses `get_file_path` and prints the saved path via `Path::display`.
- `analyze_dat_quality.sh`: discovers `.dat` files recursively with `find` instead of a flat `ls *.dat`, so it works with the new hierarchy.

## Documentation Updates

- `README.md`: the output-path description now documents the nested layout, with the letter convention updated to X = steps, Y = model, Z = dimension.
- `DATA_FORMAT.md`: the `shasum` example and the worked example use the new path and filename.
- `LIBRARY_USAGE.md`: the utility section documents `get_file_path` returning `PathBuf` and the path format.

## Testing

- Filename tests (`simulation_test.rs`, `filename_consistency.rs`) now assert the exact nested path, including the directory components, for every model.
- All 59 tests pass; `cargo fmt --check` and `cargo clippy --all-targets` are clean. Verified end to end via a CLI run: the data file and its `.sha256` land in `data/steps30/model0/`, and cleanup leaves no empty directories.
