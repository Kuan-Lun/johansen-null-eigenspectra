# Release Notes v0.10.0

## Overview

This release replaces the fixed simulation batch size with an adaptive scheme that tunes itself to the runtime environment. Instead of always processing 10,000 seeds between checkpoints, the engine now measures how long each batch actually takes and converges toward a target checkpoint interval, so the sync/progress cadence stays consistent whether the run is a small model on a laptop or a large model on a many-core server. There are no changes to the CLI, the public API, or the on-disk file format.

## New Features

### Adaptive Batch Sizing

- The parallel compute loop now times every batch and adjusts the next batch size toward a target checkpoint interval (`TARGET_BATCH_SECS`, 10 seconds). A smoothing factor (`BATCH_SMOOTHING`, 0.5) limits each correction to half the measured error, preventing oscillation from per-batch timing noise.
- The batch size is clamped to a range derived from both the machine and the task:
  - **Lower bound** — `rayon` thread count × `BATCH_FLOOR_PER_THREAD` (100), keeping every core fed and amortizing the per-batch `fsync`.
  - **Upper bound** — `num_runs / CHECKPOINT_DIVISOR` (10), guaranteeing roughly ten checkpoints and progress reports per run regardless of environment.
  - When the two bounds conflict (small runs on machines with many threads), the task-derived upper bound wins: the machine heuristic loses its meaning on a run that small, while checkpoint granularity remains a hard guarantee.
- The first batch starts at the lower bound and adapts from there; the checkpoint/resume semantics are unchanged (a power loss still costs at most the most recent batch, detected by the CRC scan and recomputed).

## Refactoring and Improvements

- `src/data_storage/config.rs`: the `BATCH_SIZE` constant is replaced by `TARGET_BATCH_SECS`, `BATCH_FLOOR_PER_THREAD`, `CHECKPOINT_DIVISOR`, and `BATCH_SMOOTHING` (all internal; the `config` module is private).
- `src/data_storage/parallel_compute.rs`: the fixed `chunks(BATCH_SIZE)` iteration is replaced by a manual slicing loop that recomputes the batch size after every batch from its measured duration.

## Testing

- All 59 tests pass; `cargo fmt --check` and `cargo clippy --all-targets` are clean.
