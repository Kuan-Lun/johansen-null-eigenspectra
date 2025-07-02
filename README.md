# Johansen Null Eigenspectra

This project performs large scale Monte Carlo simulations of the eigenvalues that appear in Johansen's null distribution. Results are written to `*.dat` files which can be reloaded later for analysis.

## Quick Start

### Option 1: Download Binary (Recommended)

1. Go to the [Releases page](https://github.com/Kuan-Lun/johansen-null-eigenspectra/releases)
2. Download the binary for your platform
3. Run the help command: `./johansen-null-eigenspectra --help`

### Option 2: Build from Source

Requires a working C compiler and LAPACK. See [BUILD.md](./BUILD.md) for compiler setup and [LAPACK_SETUP.md](./LAPACK_SETUP.md) if your system does not already provide LAPACK/BLAS.

```bash
# For source builds (all platforms)
cargo build --release

# Run the built binary:
./target/release/johansen-null-eigenspectra --help
```

## Command line interface

The binary accepts several options. The following list mirrors the help output from `src/cli.rs`:

```text
--threads <int>      number of threads for parallel computation (default: number of logical cores)
--steps <int>        number of simulation steps (default: 10,000)
--runs <int>         number of runs per model (default: 10,000,000)
--dim-start <int>    starting matrix dimension (default: 1)
--dim-end <int>      ending matrix dimension (default: 12)
--dim <int>          run a single dimension (sets start and end to the same value)
--model <list>       comma separated list of model numbers to compute (default: 0,1,2,3,4)
--quiet              suppress progress output
-h, --help           show this help message
-v, --version        show version information
```

## Usage Examples

This example runs the simulation for dimension 5 with 5,000 steps and 1,000,000 runs per model using 4 threads:

```bash
./johansen-null-eigenspectra --threads 4 --steps 5,000 --runs 1,000,000 --dim 5

# Building from source
cargo run --release -- --threads 4 --steps 5,000 --runs 1,000,000 --dim 5
```

**Note**: Numeric arguments support comma separators for better readability (e.g., `--runs 1,000,000` or `--runs 1000000`).

### Model Numbers

The `--model` parameter accepts comma-separated model numbers (0-4). Each number corresponds to a specific Johansen cointegration test model:

| Model | Description |
|-------|-------------|
| 0 | No intercept, no trend |
| 1 | Intercept, no trend, intercept in cointegration |
| 2 | Intercept, no trend, intercept not fully explained by cointegration |
| 3 | Intercept, trend, trend in cointegration |
| 4 | Intercept, trend, intercept and trend not fully explained by cointegration |

**Examples:**

- `--model 0,2` runs only models 0 and 2
- `--model 1` runs only model 1
- If not specified, all models (0,1,2,3,4) are computed by default

The simulation writes results to `data/eigenvalues_modelX_dimY_stepsZ_N.dat` where `X` is the model number, `Y` is the dimension, `Z` is the number of steps, and `N` is the number of runs.

## Data File Format

The simulation results are stored in a custom binary format optimized for high-performance writing and reading of large-scale simulation data. For detailed information about the file structure, including byte-level format specifications, see [DATA_FORMAT.md](./DATA_FORMAT.md).

**Key features of the data format:**

- Efficient append-only writing with resume capability
- Built-in data integrity verification
- Optimized for large files (millions of records)
- Cross-platform compatibility (little-endian encoding)

## Reading existing results

The library exposes `EigenvalueSimulation` in the module `johansen_null_eigenspectra::data_storage`. To use it, you need to:

1. Create an instance with the same parameters used for simulation:

```rust
let simulation = EigenvalueSimulation::new(dim, steps, num_runs);
```

1. Run the simulation (if data doesn't exist yet):

```rust
simulation.run_simulation(&models); // or run_simulation_quiet(&models)
```

1. Read the data using the method:

```rust
pub fn read_data(&self, model: JohansenModel) -> std::io::Result<Vec<(u64, Vec<f64>)>>
```

### Complete example

```rust
use johansen_null_eigenspectra::data_storage::EigenvalueSimulation;
use johansen_null_eigenspectra::johansen_models::JohansenModel;

// Create simulation with same parameters as command line: --dim 5 --steps 5000 --runs 1000000
let simulation = EigenvalueSimulation::new(5, 5000, 1000000);

// Read data for Model 0 (NoInterceptNoTrend)
match simulation.read_data(JohansenModel::NoInterceptNoTrend) {
    Ok(data) => {
        println!("Loaded {} eigenvalue records", data.len());
        for (seed, eigenvalues) in data.iter().take(5) {
            println!("Seed: {}, Eigenvalues: {:?}", seed, eigenvalues);
        }
    }
    Err(e) => eprintln!("Error reading data: {}", e),
}
```

This method reads all eigenvalue records for the given model from the generated `.dat` file. Each record contains the random seed and the corresponding eigenvalue vector. These data can be used for applications such as the Johansen trace test or maximum eigenvalue test.

**Important**: The `EigenvalueSimulation` instance must be created with the same `dim`, `steps`, and `num_runs` parameters that were used to generate the data files, as these parameters determine the filename.
