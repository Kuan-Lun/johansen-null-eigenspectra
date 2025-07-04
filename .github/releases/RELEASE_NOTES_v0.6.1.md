# Release Notes: Version 0.6.1

## Highlights

- **API Refactoring**
  - `EigenvalueSimulation` now takes a `model` argument and has an updated `new()` constructor with parameters reordered to specify the model first.
  - `run_simulation()` and `run_simulation_quiet()` only execute one model at a time; loop over models yourself if needed.
  - `run_model_simulation()` was restructured to follow SOLID principles and is now part of the `parallel_compute` module.
- **Streamlined Statistics Workflow**
  - Removed the statistics collection thread from `parallel_compute`; the `main` function now analyzes results directly.
  - `analyze_simulation_statistics` processes a single model and immediately prints statistics.
- **Testing and Structure**
  - All `#[cfg(test)]` unit tests originally under `src/` have been moved to the `tests/` directory following Rust conventions.
  - Functions in the `data_storage` sub-module were reorganized for clarity and future expansion.
- **Tools and Documentation**
  - `analyze_dat_quality.sh` has improved compression ratio calculations and messages for more accurate DAT file quality assessment.
  - Added a new `F_MATRIX.md` file and expanded the ``Theoretical Background'' section of the README to explain how each Johansen model builds its F matrix.
  - Updated the README to clarify the return type of `read_data`.

## Notes

- **Breaking Changes**
   Existing code must adapt to the new API:
  - When creating `EigenvalueSimulation`, pass the model argument.
  - To simulate multiple models, call run_simulation() in a loop.

---

**Full Changelog**: [v0.5.0...v0.6.1](https://github.com/Kuan-Lun/johansen-null-eigenspectra/compare/v0.5.0...v0.6.1)
