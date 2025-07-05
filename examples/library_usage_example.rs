use johansen_null_eigenspectra::{EigenvalueSimulation, JohansenModel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create simulation with the same parameters used by the CLI: --dim 5 --steps 1000 --runs 1000
    let simulation = EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 5, 1000, 1000);

    // Run the simulation if data does not already exist
    simulation.run_simulation();

    // Read data for model 0
    let records = simulation.read_data()?;
    println!("Loaded {} eigenvalue records", records.len());

    // Calculate percentiles of the trace statistic
    let p = vec![0.5, 0.9, 0.95];
    let trace = simulation.calculate_trace_percentiles(&p)?;
    println!("Trace percentiles: {:?}", trace);

    Ok(())
}
