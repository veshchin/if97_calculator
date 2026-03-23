pub trait ThermodynamicRegion {
    fn gamma(&self, pi: f64, tau: f64) -> f64;
    fn gamma_pi(&self, pi: f64, tau: f64) -> f64;
    fn gamma_tau(&self, pi: f64, tau: f64) -> f64;
    fn gamma_pi_pi(&self, pi: f64, tau: f64) -> f64;
    fn gamma_tau_tau(&self, pi: f64, tau: f64) -> f64;
    fn gamma_pi_tau(&self, pi: f64, tau: f64) -> f64;
}