# Library Usage

This crate exposes an API for running Johansen null eigenvalue simulations and for analysing the results. The following sections summarise the available models, basic functions and example code.

## Relationship to CLI models

The command line `--model` option uses numeric identifiers (0–4). These map directly to the `JohansenModel` enum variants:

| Model | `JohansenModel` variant | Description |
|-------|------------------------|-------------|
| 0 | `NoInterceptNoTrend` | No intercept, no trend |
| 1 | `InterceptNoTrendWithInterceptInCoint` | Intercept, no trend, intercept in cointegration |
| 2 | `InterceptNoTrendNoInterceptInCoint` | Intercept, no trend, intercept not fully explained by cointegration |
| 3 | `InterceptTrendWithTrendInCoint` | Intercept, trend, trend in cointegration |
| 4 | `InterceptTrendNoTrendInCoint` | Intercept, trend, intercept and trend not fully explained by cointegration |

`JohansenModel::default()` returns `InterceptNoTrendNoInterceptInCoint` (model 2).

## Basic functions

### Creating a simulation

```rust
pub fn new(model: JohansenModel, dim: usize, steps: usize, num_runs: usize) -> Self
```
Creates a configuration for running simulations or reading data.

### Running simulations

```rust
pub fn run_simulation(&self)
pub fn run_simulation_quiet(&self)
```
Start the Monte Carlo computation for the configured model. The *quiet* variant suppresses progress output.

### Reading data

```rust
pub fn read_all_data(&self) -> std::io::Result<Vec<(u32, Vec<f64>)>>
pub fn read_data(&self) -> std::io::Result<Vec<(u32, Vec<f64>)>>
```
`read_all_data` returns every record found in the data file, while `read_data` restricts the output to `num_runs` records and reports an error if fewer are available.

### Calculating percentiles

```rust
pub fn calculate_trace_percentiles(&self, p: &[f64]) -> Result<Vec<f64>, Box<dyn std::error::Error>>
pub fn calculate_maxeig_percentiles(&self, p: &[f64]) -> Result<Vec<f64>, Box<dyn std::error::Error>>
```
Compute percentiles for the trace statistic or the maximum eigenvalue statistic using the values returned by `read_data`.

### Utility

```rust
pub fn get_filename(&self, model: JohansenModel) -> String
```
Return the file name used to store simulation results for a particular model.

## Example

See `examples/library_usage_example.rs` for a complete runnable example.

```rust
use johansen_null_eigenspectra::{EigenvalueSimulation, JohansenModel};

// Create simulation with the same parameters used by the CLI: --dim 5 --steps 5000 --runs 1000000
let simulation = EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 5, 5000, 1_000_000);

// Run the simulation if data does not already exist
simulation.run_simulation();

// Read data for model 0
let records = simulation.read_data()?;
println!("Loaded {} eigenvalue records", records.len());

// Calculate percentiles of the trace statistic
let p = vec![0.5, 0.9, 0.95];
let trace = simulation.calculate_trace_percentiles(&p)?;
println!("Trace percentiles: {:?}", trace);
```
