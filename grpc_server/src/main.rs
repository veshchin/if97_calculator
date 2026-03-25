pub mod api;
pub mod telemetry;

use api::grpc_service::If97Service;
use api::grpc_service::pb::water_properties_server::WaterPropertiesServer;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Инициализируем систему логирования через локальный модуль
    telemetry::init_subscriber();

    // Слушаем на localhost, порт 50051
    let addr = "[::1]:50051".parse()?;
    let service = If97Service::default();

    info!("IAPWS-IF97 gRPC Core Server is running on {}", addr);

    Server::builder()
        .add_service(WaterPropertiesServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}