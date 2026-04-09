//! CLI для прямой проверки gRPC Health API (`grpc.health.v1.Health`).
//!
//! В отличие от `grpcurl`/`grpc_health_probe`, утилита не требует сторонних бинарников:
//! она использует `tonic-health` и может запускаться в CI вместе с сервисом.

use tonic_health::pb::health_check_response::ServingStatus;
use tonic_health::pb::health_client::HealthClient;
use tonic_health::pb::HealthCheckRequest;

#[derive(Clone)]
struct Opts {
    addr: String,
    service: String,
    io_threads: usize,
    max_msg_bytes: usize,
}

fn parse_usize(arg: Option<&String>) -> Option<usize> {
    arg.and_then(|s| s.parse::<usize>().ok())
}

fn parse_args() -> Opts {
    let mut addr: Option<String> = None;
    let mut service: Option<String> = None;
    let mut io_threads: Option<usize> = None;
    let mut max_msg_bytes: Option<usize> = None;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--addr" => {
                addr = args.get(i + 1).cloned();
                i += 2;
            }
            "--service" => {
                service = args.get(i + 1).cloned();
                i += 2;
            }
            "--io-threads" => {
                io_threads = parse_usize(args.get(i + 1));
                i += 2;
            }
            "--max-msg-bytes" => {
                max_msg_bytes = parse_usize(args.get(i + 1));
                i += 2;
            }
            _ => i += 1,
        }
    }

    Opts {
        addr: addr
            .or_else(|| std::env::var("IF97_GRPC_ADDR").ok())
            .unwrap_or_else(|| "127.0.0.1:50051".to_string()),
        service: service.unwrap_or_else(|| "if97.v1.If97Service".to_string()),
        io_threads: io_threads
            .or_else(|| std::env::var("IF97_HC_IO_THREADS").ok()?.parse().ok())
            .unwrap_or(1)
            .max(1),
        max_msg_bytes: max_msg_bytes
            .or_else(|| std::env::var("IF97_HC_MAX_MSG_BYTES").ok()?.parse().ok())
            .unwrap_or(4 * 1024 * 1024)
            .max(1),
    }
}

async fn check(
    client: &mut HealthClient<tonic::transport::Channel>,
    addr: &str,
    service: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client
        .check(HealthCheckRequest {
            service: service.to_string(),
        })
        .await?
        .into_inner();

    let status =
        ServingStatus::try_from(resp.status).unwrap_or(ServingStatus::Unknown);

    println!(
        "grpc_healthcheck: addr={} service=\"{}\" status={:?}",
        addr, service, status
    );

    if status != ServingStatus::Serving {
        return Err(format!(
            "health check failed: service={service:?} status={status:?}"
        )
        .into());
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = parse_args();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(opts.io_threads)
        .enable_all()
        .build()?;

    rt.block_on(async move {
        let endpoint = format!("http://{}", opts.addr);
        let channel = tonic::transport::Channel::from_shared(endpoint)?
            .connect()
            .await?;

        let mut client = HealthClient::new(channel)
            .max_decoding_message_size(opts.max_msg_bytes)
            .max_encoding_message_size(opts.max_msg_bytes);

        // 1) По имени сервиса (как в `tonic_health::server::HealthReporter::set_service_status`)
        check(&mut client, &opts.addr, &opts.service).await?;
        // 2) Пустое имя сервиса трактуется как общий статус сервера.
        check(&mut client, &opts.addr, "").await?;

        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    Ok(())
}

