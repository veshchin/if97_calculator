//! CLI для нагрузочного тестирования gRPC API `If97Service/CalculatePt`.
//!
//! Утилита генерирует поток батчей `p[]/t[]` и измеряет throughput/latency.

use if97_calculator_service::pb::if97::if97_service_client::If97ServiceClient;
use if97_calculator_service::pb::if97::PtBatchRequest;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone)]
struct Opts {
    addr: String,
    streams: usize,
    batches: usize,
    batch_size: usize,
    in_flight: usize,
    p: f64,
    t: f64,
    io_threads: usize,
    max_msg_bytes: usize,
}

fn parse_usize(arg: Option<&String>) -> Option<usize> {
    arg.and_then(|s| s.parse::<usize>().ok())
}

fn parse_f64(arg: Option<&String>) -> Option<f64> {
    arg.and_then(|s| s.parse::<f64>().ok())
}

fn parse_args() -> Opts {
    let mut addr: Option<String> = None;
    let mut streams: Option<usize> = None;
    let mut batches: Option<usize> = None;
    let mut batch_size: Option<usize> = None;
    let mut in_flight: Option<usize> = None;
    let mut p: Option<f64> = None;
    let mut t: Option<f64> = None;
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
            "--streams" => {
                streams = parse_usize(args.get(i + 1));
                i += 2;
            }
            "--batches" => {
                batches = parse_usize(args.get(i + 1));
                i += 2;
            }
            "--batch-size" => {
                batch_size = parse_usize(args.get(i + 1));
                i += 2;
            }
            "--in-flight" => {
                in_flight = parse_usize(args.get(i + 1));
                i += 2;
            }
            "--p" => {
                p = parse_f64(args.get(i + 1));
                i += 2;
            }
            "--t" => {
                t = parse_f64(args.get(i + 1));
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

    let streams = streams
        .or_else(|| std::env::var("IF97_LT_STREAMS").ok()?.parse().ok())
        .unwrap_or(1);
    let batches = batches
        .or_else(|| std::env::var("IF97_LT_BATCHES").ok()?.parse().ok())
        .unwrap_or(2000);
    let batch_size = batch_size
        .or_else(|| std::env::var("IF97_LT_BATCH_SIZE").ok()?.parse().ok())
        .unwrap_or(2048);
    let in_flight = in_flight
        .or_else(|| std::env::var("IF97_LT_IN_FLIGHT").ok()?.parse().ok())
        .unwrap_or(8);
    let p = p
        .or_else(|| std::env::var("IF97_LT_P").ok()?.parse().ok())
        .unwrap_or(10.0);
    let t = t
        .or_else(|| std::env::var("IF97_LT_T").ok()?.parse().ok())
        .unwrap_or(500.0);
    let io_threads = io_threads
        .or_else(|| std::env::var("IF97_LT_IO_THREADS").ok()?.parse().ok())
        .unwrap_or(2);
    let max_msg_bytes = max_msg_bytes
        .or_else(|| std::env::var("IF97_LT_MAX_MSG_BYTES").ok()?.parse().ok())
        .unwrap_or(32 * 1024 * 1024);

    Opts {
        addr: addr
            .or_else(|| std::env::var("IF97_GRPC_ADDR").ok())
            .unwrap_or_else(|| "127.0.0.1:50051".to_string()),
        streams: streams.max(1),
        batches: batches.max(1),
        batch_size: batch_size.max(1),
        in_flight: in_flight.max(1),
        p,
        t,
        io_threads: io_threads.max(1),
        max_msg_bytes: max_msg_bytes.max(1),
    }
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

        let start = Instant::now();
        let mut handles = Vec::with_capacity(opts.streams);

        for stream_id in 0..opts.streams {
            let mut client = If97ServiceClient::new(channel.clone())
                .max_decoding_message_size(opts.max_msg_bytes)
                .max_encoding_message_size(opts.max_msg_bytes);

            let (req_tx, req_rx) = mpsc::channel::<PtBatchRequest>(opts.in_flight);
            let mut resp = client
                .calculate_pt(ReceiverStream::new(req_rx))
                .await?
                .into_inner();

            let send_opts = opts.clone();
            let sender = tokio::spawn(async move {
                for batch_idx in 0..send_opts.batches {
                    let batch_id = ((stream_id as u64) << 32) | (batch_idx as u64);
                    let req = PtBatchRequest {
                        batch_id,
                        p: vec![send_opts.p; send_opts.batch_size],
                        t: vec![send_opts.t; send_opts.batch_size],
                    };
                    if req_tx.send(req).await.is_err() {
                        break;
                    }
                }
            });

            let recv_opts = opts.clone();
            let receiver = tokio::spawn(async move {
                let mut received: usize = 0;
                while received < recv_opts.batches {
                    match resp.message().await {
                        Ok(Some(_)) => received += 1,
                        Ok(None) => break,
                        Err(status) => return Err(status),
                    }
                }
                Ok::<usize, tonic::Status>(received)
            });

            handles.push((sender, receiver));
        }

        let mut total_batches: usize = 0;
        for (sender, receiver) in handles {
            let _ = sender.await;
            total_batches += receiver.await??;
        }

        let elapsed = start.elapsed();
        let secs = elapsed.as_secs_f64().max(1e-9);
        let total_points = (total_batches as f64) * (opts.batch_size as f64);

        println!(
            "grpc_loadtest: addr={} streams={} batches={} batch_size={} total_batches={} elapsed={:.3}s batches/s={:.0} points/s={:.0}",
            opts.addr,
            opts.streams,
            opts.batches,
            opts.batch_size,
            total_batches,
            secs,
            (total_batches as f64) / secs,
            total_points / secs
        );

        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    Ok(())
}
