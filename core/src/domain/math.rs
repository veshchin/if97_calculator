pub trait GibbsRegion {
    fn gamma(&self, pi: f64, tau: f64) -> f64;
    fn gamma_pi(&self, pi: f64, tau: f64) -> f64;
    fn gamma_tau(&self, pi: f64, tau: f64) -> f64;
    fn gamma_pi_pi(&self, pi: f64, tau: f64) -> f64;
    fn gamma_tau_tau(&self, pi: f64, tau: f64) -> f64;
    fn gamma_pi_tau(&self, pi: f64, tau: f64) -> f64;
}

pub trait HelmholtzRegion {
    fn phi(&self, delta: f64, tau: f64) -> f64;
    fn phi_delta(&self, delta: f64, tau: f64) -> f64;
    fn phi_delta_delta(&self, delta: f64, tau: f64) -> f64;
    fn phi_tau(&self, delta: f64, tau: f64) -> f64;
    fn phi_tau_tau(&self, delta: f64, tau: f64) -> f64;
    fn phi_delta_tau(&self, delta: f64, tau: f64) -> f64;
}