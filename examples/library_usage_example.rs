use johansen_null_eigenspectra::{EigenvalueSimulation, Error, JohansenModel, Statistic};

fn main() -> Result<(), Error> {
    // Create a simulation configuration: model 0, dim 5, 1000 steps, 1000 runs
    let simulation = EigenvalueSimulation::new(JohansenModel::NoInterceptNoTrend, 5, 1000, 1000)?;

    // Run the simulation, resuming from existing data when possible.
    // The library never prints; progress arrives through the callback.
    simulation.run_with_progress(|progress| {
        println!("progress: {}/{}", progress.completed, progress.total);
    })?;

    // Load every record into memory (seeds 1..=num_runs, in seed order)
    let data = simulation.load()?;
    println!("Loaded {} eigenvalue records", data.len());

    // Compute percentiles of the trace and max-eigenvalue statistics
    let p = [0.5, 0.9, 0.95];
    let trace = data.percentiles(Statistic::Trace, &p);
    let maxeig = data.percentiles(Statistic::MaxEigenvalue, &p);
    println!("Trace percentiles: {trace:?}");
    println!("MaxEig percentiles: {maxeig:?}");

    Ok(())
}
