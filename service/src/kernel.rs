use if97_core::errors::If97Error;
use if97_core::If97;
use rayon::prelude::*;

pub trait Kernel: Send + Sync + 'static {
    fn h_pt_batch(&self, p: &[f64], t: &[f64]) -> (Vec<f64>, Vec<u32>);
}

pub struct StubKernel;
pub struct CoreKernel;

fn map_if97_error(err: If97Error) -> u32 {
    match err {
        If97Error::OutOfBounds(_) => 1,
        If97Error::PhaseBoundaryError(_) => 2,
        _ => 3,
    }
}

impl Kernel for StubKernel {
    fn h_pt_batch(&self, p: &[f64], t: &[f64]) -> (Vec<f64>, Vec<u32>) {
        let n = p.len();
        let mut h = vec![0.0; n];
        let mut status = vec![0u32; n];

        h.par_iter_mut()
            .zip(status.par_iter_mut())
            .zip(p.par_iter().zip(t.par_iter()))
            .for_each(|((h_out, st_out), (&p, &t))| {
                *h_out = p + t;
                *st_out = 0;
            });

        (h, status)
    }
}

impl Kernel for CoreKernel {
    fn h_pt_batch(&self, p: &[f64], t: &[f64]) -> (Vec<f64>, Vec<u32>) {
        let n = p.len();
        let mut h = vec![f64::NAN; n];
        let mut status = vec![0u32; n];

        h.par_iter_mut()
            .zip(status.par_iter_mut())
            .zip(p.par_iter().zip(t.par_iter()))
            .for_each(|((h_out, st_out), (&p, &t))| match If97::pt(p.into(), t.into()) {
                Ok(state) => {
                    *h_out = state.h.inner();
                    *st_out = 0;
                }
                Err(err) => {
                    *h_out = f64::NAN;
                    *st_out = map_if97_error(err);
                }
            });

        (h, status)
    }
}

