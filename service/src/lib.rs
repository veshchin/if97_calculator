pub mod config;
pub mod pb;
pub mod telemetry;

mod engine;
mod grpc;
mod kernel;
mod shutdown;

use config::Config;
use std::sync::Arc;

pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let cpu_threads = config.cpu_threads.max(1);

    rayon::ThreadPoolBuilder::new()
        .num_threads(cpu_threads)
        .thread_name(|i| format!("if97-cpu-{i}"))
        .build_global()?;

    let io_threads = config.io_threads.max(1);
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(io_threads)
        .enable_all()
        .build()?;

    rt.block_on(async move {
        tracing::info!("if97_calculator_service listening on {}", config.grpc_addr);

        let kernel: Arc<dyn kernel::Kernel> = match config.kernel {
            config::KernelKind::Core => Arc::new(kernel::CoreKernel),
            config::KernelKind::Stub => Arc::new(kernel::StubKernel),
        };

        let dispatcher =
            engine::Dispatcher::new(kernel, config.max_in_flight_global.max(1));
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let svc = grpc::If97Grpc::new(config.clone(), dispatcher, shutdown_rx);

        let (mut health_reporter, health_service) =
            tonic_health::server::health_reporter();
        health_reporter
            .set_service_status(
                "if97.v1.If97Service",
                tonic_health::ServingStatus::Serving,
            )
            .await;
        health_reporter
            .set_service_status(
                "",
                tonic_health::ServingStatus::Serving,
            )
            .await;

        let service = pb::if97::if97_service_server::If97ServiceServer::new(svc)
            .max_decoding_message_size(config.max_msg_bytes)
            .max_encoding_message_size(config.max_msg_bytes);

        tonic::transport::Server::builder()
            .tcp_nodelay(true)
            .tcp_keepalive(Some(std::time::Duration::from_secs(600)))
            .http2_keepalive_interval(Some(std::time::Duration::from_secs(60)))
            .http2_keepalive_timeout(Some(std::time::Duration::from_secs(10)))
            .http2_adaptive_window(Some(true))
            .add_service(health_service)
            .add_service(service)
            .serve_with_shutdown(config.grpc_addr, async move {
                shutdown::shutdown_signal().await;
                let _ = shutdown_tx.send(true);
                health_reporter
                    .set_service_status(
                        "if97.v1.If97Service",
                        tonic_health::ServingStatus::NotServing,
                    )
                    .await;
                health_reporter
                    .set_service_status(
                        "",
                        tonic_health::ServingStatus::NotServing,
                    )
                    .await;
                if config.drain_secs > 0 {
                    tokio::time::sleep(std::time::Duration::from_secs(
                        config.drain_secs,
                    ))
                    .await;
                }
            })
            .await?;

        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    Ok(())
}
