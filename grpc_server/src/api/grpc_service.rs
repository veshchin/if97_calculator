use tonic::{Request, Response, Status};
use if97_core::domain::calculator::Calculator;
use if97_core::domain::state::WaterState;
use if97_core::app::router::Router;

pub mod pb {
    tonic::include_proto!("if97");
}

use pb::water_properties_server::WaterProperties;
use pb::{PointPt, PointPh, PointPs, PointPx, WaterStateResponse};

#[derive(Default)]
pub struct If97Service;

impl From<WaterState> for WaterStateResponse {
    fn from(state: WaterState) -> Self {
        WaterStateResponse {
            p: state.p,
            t: state.t,
            v: state.v,
            rho: state.rho,
            h: state.h,
            s: state.s,
            cp: state.cp,
            w: state.w,
            region: match state.region {
                if97_core::domain::state::Region::Region1 => 1,
                if97_core::domain::state::Region::Region2 => 2,
                if97_core::domain::state::Region::Region3 => 3,
                if97_core::domain::state::Region::Region4 => 4,
                if97_core::domain::state::Region::Region5 => 5,
                if97_core::domain::state::Region::OutOfBounds => -1,
            },
        }
    }
}

fn map_err(e: &'static str) -> Status {
    Status::invalid_argument(e)
}

#[tonic::async_trait]
impl WaterProperties for If97Service {

    #[tracing::instrument(skip(self))]
    async fn calculate_pt(
        &self,
        request: Request<PointPt>,
    ) -> Result<Response<WaterStateResponse>, Status> {
        let req = request.into_inner();
        let state = Calculator::calculate_pt(req.p, req.t).map_err(map_err)?;
        Ok(Response::new(state.into()))
    }

    #[tracing::instrument(skip(self))]
    async fn calculate_ph(
        &self,
        request: Request<PointPh>,
    ) -> Result<Response<WaterStateResponse>, Status> {
        let req = request.into_inner();
        let state = Router::calculate_ph(req.p, req.h).map_err(map_err)?;
        Ok(Response::new(state.into()))
    }

    #[tracing::instrument(skip(self))]
    async fn calculate_ps(
        &self,
        request: Request<PointPs>
    ) -> Result<Response<WaterStateResponse>, Status> {
        let req = request.into_inner();
        let state = Router::calculate_ps(req.p, req.s).map_err(map_err)?;
        Ok(Response::new(state.into()))
    }

    #[tracing::instrument(skip(self))]
    async fn calculate_two_phase(
        &self,
        request: Request<PointPx>
    ) -> Result<Response<WaterStateResponse>, Status> {
        let req = request.into_inner();
        let state = Calculator::calculate_px(req.p, req.x).map_err(map_err)?;
        Ok(Response::new(state.into()))
    }
}