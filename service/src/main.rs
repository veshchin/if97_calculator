fn main() -> Result<(), Box<dyn std::error::Error>> {
    if97_calculator_service::telemetry::init();
    let config = if97_calculator_service::config::Config::from_env()?;
    if97_calculator_service::run(config)
}
