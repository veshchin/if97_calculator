#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Region {
    #[default]
    OutOfBounds,
    Region1, Region2, Region3, Region4, Region5,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub struct WaterState {
    pub p: f64, pub t: f64, pub v: f64, pub rho: f64,
    pub h: f64, pub s: f64, pub cp: f64, pub w: f64, pub region: Region,
}