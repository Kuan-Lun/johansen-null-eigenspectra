# Library Usage

This crate exposes an API for running Johansen null eigenvalue simulations and for analysing the results. The public surface is intentionally small:

| Type | Role |
| ---- | ---- |
| `EigenvalueSimulation` | The entry point: a validated simulation configuration that can run, load, verify, and locate its data |
| `SimulationData` | Simulation results loaded into memory; computes test-statistic percentiles |
| `SimulationRecord` | One record: a `seed` and its `eigenvalues` |
| `Statistic` | Which test statistic to aggregate: `Trace` or `MaxEigenvalue` |
| `Progress` | A progress report emitted while a simulation runs |
| `JohansenModel` | The five Johansen model specifications |
| `Error` | The crate-wide error type |

Design principles: the library never prints, never panics on invalid input, and never deletes files on its own — those responsibilities belong to the caller (the bundled CLI is one such caller).

## `JohansenModel`

The five model specifications map directly to the numeric identifiers used by the CLI's `--model` option:

| Number | Variant | Description |
| ------ | ------- | ----------- |
| 0 | `NoInterceptNoTrend` | No intercept, no trend |
| 1 | `InterceptNoTrendWithInterceptInCoint` | Intercept, no trend, intercept in cointegration |
| 2 | `InterceptNoTrendUnrestrictedIntercept` | Intercept, no trend, intercept not fully explained by cointegration |
| 3 | `InterceptTrendUnrestrictedInterceptRestrictedTrend` | Intercept, trend, trend in cointegration |
| 4 | `InterceptTrendUnrestrictedBoth` | Intercept, trend, intercept and trend not fully explained by cointegration |

```rust
JohansenModel::ALL                  // [JohansenModel; 5], ordered by model number
u8::from(model)                     // model -> number (0-4)
JohansenModel::try_from(3)?         // number -> model; Err(Error::InvalidConfig) outside 0..=4
model.eigenvalues_per_run(dim)      // dim + 1 for models 1 and 3, dim otherwise
JohansenModel::default()            // model 2, the most common specification
println!("{model}")                 // "Model 2: Intercept, no trend, ..."
```

The enum is a mathematically closed set and is deliberately **not** `#[non_exhaustive]`: exhaustive `match` over the five variants is supported.

## `EigenvalueSimulation`

### Creating a configuration

```rust
pub fn new(model: JohansenModel, dim: usize, steps: usize, num_runs: usize)
    -> Result<EigenvalueSimulation, Error>
```

Validates the storable range and returns `Error::InvalidConfig` otherwise: `dim` in `1..=254`, `steps` and `num_runs` in `1..=u32::MAX`. Fields are private; read them back with the `model()`, `dim()`, `steps()`, `num_runs()` getters.

```rust
pub fn with_data_dir(self, dir: impl Into<PathBuf>) -> Self
```

Replaces the data root directory (default: `data/` under the working directory). The rest of the layout is fixed; `data_path()` returns the resulting file path:

```rust
pub fn data_path(&self) -> PathBuf
// {data_dir}/steps{steps}/model{model}/eigenvalues_steps{steps}_model{model}_dim{dim}.dat
```

`num_runs` is intentionally absent from the path: the same file is expanded in place when a larger `num_runs` is requested later.

### Running

```rust
pub fn run(&self) -> Result<(), Error>
pub fn run_with_progress(&self, on_progress: impl FnMut(Progress)) -> Result<(), Error>
```

Runs the Monte Carlo computation for the configured model, resuming from any valid records already on disk. `run` is silent; `run_with_progress` reports through the callback instead of printing:

- The first `Progress { completed, total }` arrives right after the file is opened and describes the resume point (`completed == total` means everything was already done).
- Subsequent reports arrive after each batch is flushed to disk.

If an existing file at `data_path()` is incompatible (wrong format, or a conflicting configuration), the run returns an error **without touching the file**. Since all data can be recomputed from seeds, the caller may choose to delete and recompute:

```rust
match simulation.run() {
    Ok(()) => {}
    Err(e) if e.is_recreatable() => {
        simulation.delete_data();   // explicit, destructive
        simulation.run()?;
    }
    Err(e) => return Err(e),
}
```

```rust
pub fn delete_data(&self)
```

Removes the data file and its sidecar files (CRC and SHA-256), then removes the containing `model{}/` and `steps{}/` directories if they became empty. Destructive; the data is recomputable but the compute time is not refunded.

### Loading results

```rust
pub fn load(&self) -> Result<SimulationData, Error>
pub fn load_partial(&self) -> Result<SimulationData, Error>
```

`load` returns the records for seeds `1..=num_runs` in seed order and fails with `Error::Incomplete` if any record is missing — use it when the analysis requires the full sample. `load_partial` returns every valid record currently stored, skipping gaps — use it to inspect an interrupted simulation. Both return `SimulationData`; `is_complete()` tells whether all requested seeds are present.

### Verifying integrity

```rust
pub fn verify(&self) -> Result<(), Error>
```

Recomputes the SHA-256 hash of a completed data file and compares it against the accompanying `.sha256` file. Intended for files received from others or retrieved from long-term storage; ordinary reads do not pay the whole-file hashing cost. The hash file uses the standard `sha256sum` format, so it can also be checked externally with `shasum -a 256 -c`.

## `SimulationData`

```rust
data.records()          // &[SimulationRecord], in seed order
data.is_complete()      // all requested seeds 1..=num_runs present?
data.len() / data.is_empty()
data.percentiles(Statistic::Trace, &[0.90, 0.95, 0.99])   // -> Vec<f64>
```

`percentiles` aggregates each record's eigenvalues into the chosen test statistic (`Trace` sums them, `MaxEigenvalue` takes the largest), sorts the resulting sample, and evaluates each requested percentile (`0.0..=1.0`) with linear interpolation. The data is already in memory, so the call is infallible and can be repeated cheaply for different statistics or percentile sets; the returned vector matches the input length (NaN per entry when there are no records).

For any statistic beyond the built-in two, work directly on `records()` — each `SimulationRecord` exposes its `seed` and `eigenvalues` as public fields.

## `Error`

A single crate-wide enum implementing `std::error::Error`, marked `#[non_exhaustive]` (keep a wildcard arm when matching). Notable variants: `Io`, `MagicMismatch`, `ParamMismatch`, `Corrupted`, `Incomplete`, `HashMismatch`, `InvalidInput`, `InvalidConfig`.

```rust
pub fn is_recreatable(&self) -> bool
```

Returns `true` when the error means "the existing file cannot be used, but deleting it and recomputing would succeed" (`MagicMismatch`, `ParamMismatch`, `Corrupted`).

## Example

See `examples/library_usage_example.rs` for the runnable version.

```rust
use johansen_null_eigenspectra::{EigenvalueSimulation, Error, JohansenModel, Statistic};

fn main() -> Result<(), Error> {
    let simulation =
        EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 5, 5000, 1_000_000)?;

    // Resumes from existing data; progress arrives through the callback.
    simulation.run_with_progress(|p| println!("progress: {}/{}", p.completed, p.total))?;

    let data = simulation.load()?;
    let trace = data.percentiles(Statistic::Trace, &[0.5, 0.9, 0.95]);
    let maxeig = data.percentiles(Statistic::MaxEigenvalue, &[0.5, 0.9, 0.95]);
    println!("Trace percentiles: {trace:?}");
    println!("MaxEig percentiles: {maxeig:?}");
    Ok(())
}
```
