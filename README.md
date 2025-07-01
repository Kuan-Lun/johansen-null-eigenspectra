# Johansen Null Eigenspectra

This project performs large scale Monte Carlo simulations of the eigenvalues that appear in Johansen's null distribution. Results are written to `*.dat` files which can be reloaded later for analysis.

To build the project you need a working C compiler and LAPACK. See [BUILD.md](./BUILD.md) for compiler setup and [LAPACK_SETUP.md](./LAPACK_SETUP.md) if your system does not already provide LAPACK/BLAS.

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
```

A typical invocation is:

```bash
cargo run --release -- --threads 4 --steps 5,000 --runs 1,000,000 --dim 5
```

This example runs the simulation for dimension 5 with 5,000 steps and 1,000,000 runs per model using 4 threads.

**Note**: Numeric arguments support comma separators for better readability (e.g., `--runs 1,000,000` or `--runs 1000000`).

The simulation writes results to `data/eigenvalues_modelX_dimY_stepsZ_N.dat` where `X` is the model number, `Y` is the dimension, `Z` is the number of steps, and `N` is the number of runs.

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

// Read data for Model 0
match simulation.read_data(JohansenModel::Model0) {
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
