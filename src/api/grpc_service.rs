use tonic::{Request, Response, Status};
use crate::core::calculator::Calculator;
use crate::core::state::WaterState;

// Импортируем сгенерированный код из protobuf
pub mod pb {
    tonic::include_proto!("if97");
}

use pb::water_properties_server::WaterProperties;
use pb::{PointPt, PointPh, PointPs, PointPx, WaterStateResponse};

#[derive(Default)]
pub struct If97Service;

// Вспомогательная функция для маппинга доменной модели в gRPC ответ
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
            // Маппим enum Region в int
            region: match state.region {
                crate::core::state::Region::Region1 => 1,
                crate::core::state::Region::Region2 => 2,
                crate::core::state::Region::Region3 => 3,
                crate::core::state::Region::Region4 => 4,
                crate::core::state::Region::Region5 => 5,
                crate::core::state::Region::OutOfBounds => -1,
            },
        }
    }
}

// Вспомогательная функция для маппинга ошибок
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

        let state = Calculator::calculate(req.p, req.t)
            .map_err(map_err)?;

        Ok(Response::new(state.into()))
    }

    #[tracing::instrument(skip(self))]
    async fn calculate_ph(
        &self,
        request: Request<PointPh>,
    ) -> Result<Response<WaterStateResponse>, Status> {
        let req = request.into_inner();

        // Здесь требуется слой маршрутизации, так как у тебя функции
        // разбиты по регионам (calc_region1_ph, calc_region2_ph и т.д.).
        // Для примера вызываем некую абстрактную функцию-агрегатор из слоя App
        let state = crate::app::use_cases::calculate_by_ph(req.p, req.h)
            .map_err(map_err)?;

        Ok(Response::new(state.into()))
    }

    // ... Аналогично реализуются calculate_ps и calculate_two_phase ...
    async fn calculate_ps(&self, _request: Request<PointPs>) -> Result<Response<WaterStateResponse>, Status> {
        Err(Status::unimplemented("Not implemented yet"))
    }

    async fn calculate_two_phase(&self, request: Request<PointPx>) -> Result<Response<WaterStateResponse>, Status> {
        let req = request.into_inner();
        let state = Calculator::calculate_two_phase(req.p, req.x)
            .map_err(map_err)?;

        Ok(Response::new(state.into()))
    }
}