// Test the corrected README.md API as an external user would
use johansen_null_eigenspectra::{EigenvalueSimulation, JohansenModel};

fn main() {
    // Create simulation with same parameters as command line: --dim 5 --steps 5000 --runs 1000000
    let simulation = EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 5, 5000, 1000000);

    // Read data for Model 0 (NoInterceptNoTrend)
    match simulation.read_data() {
        Ok(data) => {
            println!("Loaded {} eigenvalue records", data.len());
            for (seed, eigenvalues) in data.iter().take(5) {
                println!("Seed: {}, Eigenvalues: {:?}", seed, eigenvalues);
            }
        }
        Err(e) => eprintln!("Error reading data: {}", e),
    }
}
