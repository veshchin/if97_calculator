use crate::config::Config;
use crate::engine::Dispatcher;
use crate::pb;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

#[derive(Clone)]
pub struct If97Grpc {
    config: Config,
    dispatcher: Dispatcher,
    shutdown: tokio::sync::watch::Receiver<bool>,
}

impl If97Grpc {
    pub fn new(
        config: Config,
        dispatcher: Dispatcher,
        shutdown: tokio::sync::watch::Receiver<bool>,
    ) -> Self {
        Self {
            config,
            dispatcher,
            shutdown,
        }
    }
}

#[tonic::async_trait]
impl pb::if97::if97_service_server::If97Service for If97Grpc {
    type CalculatePtStream = ReceiverStream<Result<pb::if97::PtBatchResponse, Status>>;

    async fn calculate_pt(
        &self,
        request: Request<Streaming<pb::if97::PtBatchRequest>>,
    ) -> Result<Response<Self::CalculatePtStream>, Status> {
        let mut inbound = request.into_inner();

        let max_in_flight = self.config.max_in_flight_per_conn.max(1);
        let (tx, rx) = mpsc::channel::<Result<pb::if97::PtBatchResponse, Status>>(max_in_flight);
        let permits = Arc::new(Semaphore::new(max_in_flight));

        let dispatcher = self.dispatcher.clone();
        let config = self.config.clone();
        let mut shutdown = self.shutdown.clone();

        tokio::spawn({
            let permits = permits.clone();
            async move {
                loop {
                    if tx.is_closed() {
                        break;
                    }

                    let batch = tokio::select! {
                        _ = shutdown.changed() => {
                            if *shutdown.borrow() {
                                break;
                            }
                            continue;
                        }
                        res = inbound.message() => {
                            match res {
                                Ok(Some(batch)) => batch,
                                Ok(None) => break,
                                Err(status) => {
                                    let _ = tx.send(Err(status)).await;
                                    break;
                                }
                            }
                        }
                    };

                    let batch_id = batch.batch_id;
                    let n = batch.p.len();

                    if batch.t.len() != n {
                        let _ = tx
                            .send(Ok(pb::if97::PtBatchResponse {
                                batch_id,
                                h: Vec::new(),
                                status: Vec::new(),
                                batch_error_code: 1,
                                batch_error_message: format!(
                                    "length mismatch: len(p)={} len(t)={}",
                                    n,
                                    batch.t.len()
                                ),
                            }))
                            .await;
                        continue;
                    }

                    if n > config.max_batch_len {
                        let _ = tx
                            .send(Ok(pb::if97::PtBatchResponse {
                                batch_id,
                                h: Vec::new(),
                                status: Vec::new(),
                                batch_error_code: 2,
                                batch_error_message: format!(
                                    "batch too large: n={} max={}",
                                    n, config.max_batch_len
                                ),
                            }))
                            .await;
                        continue;
                    }

                    let conn_permit = match permits.clone().acquire_owned().await {
                        Ok(permit) => permit,
                        Err(_) => break,
                    };

                    dispatcher.dispatch_pt(batch, tx.clone(), conn_permit).await;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, KernelKind};
    use crate::kernel::{Kernel, StubKernel};
    use std::net::SocketAddr;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::oneshot;
    use tokio_stream::wrappers::TcpListenerStream;

    async fn wait_for_tcp(addr: SocketAddr) {
        for _ in 0..50 {
            if tokio::net::TcpStream::connect(addr).await.is_ok() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("server did not become ready: {addr}");
    }

    async fn spawn_server(
        config: Config,
        kernel: Arc<dyn Kernel>,
    ) -> Option<(
        SocketAddr,
        oneshot::Sender<()>,
        tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
    )> {
        let dispatcher = crate::engine::Dispatcher::new(
            kernel,
            config.max_in_flight_global.max(1),
        );
        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let svc = If97Grpc::new(config.clone(), dispatcher, shutdown_rx);

        let (mut health_reporter, health_service) =
            tonic_health::server::health_reporter();
        health_reporter
            .set_service_status(
                "if97.v1.If97Service",
                tonic_health::ServingStatus::Serving,
            )
            .await;
        health_reporter
            .set_service_status("", tonic_health::ServingStatus::Serving)
            .await;

        let service = pb::if97::if97_service_server::If97ServiceServer::new(svc)
            .max_decoding_message_size(config.max_msg_bytes)
            .max_encoding_message_size(config.max_msg_bytes);

        let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await
        {
            Ok(listener) => listener,
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                eprintln!(
                    "skipping gRPC integration tests: cannot bind sockets in this environment: {err}"
                );
                return None;
            }
            Err(err) => panic!("bind test listener: {err}"),
        };
        let addr = listener.local_addr().expect("listener local addr");
        let incoming = TcpListenerStream::new(listener);

        let (server_shutdown_tx, server_shutdown_rx) = oneshot::channel::<()>();
        let handle = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(health_service)
                .add_service(service)
                .serve_with_incoming_shutdown(incoming, async move {
                    let _ = server_shutdown_rx.await;
                })
                .await
        });

        Some((addr, server_shutdown_tx, handle))
    }

    fn test_config(max_batch_len: usize) -> Config {
        Config {
            grpc_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            io_threads: 2,
            cpu_threads: 2,
            max_in_flight_per_conn: 2,
            max_in_flight_global: 2,
            max_batch_len,
            max_msg_bytes: 4 * 1024 * 1024,
            drain_secs: 0,
            kernel: KernelKind::Stub,
        }
    }

    #[test]
    fn grpc_calculate_pt_valid_batch() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .expect("tokio runtime");

        rt.block_on(async move {
            let config = test_config(1024);
            let Some((addr, shutdown_tx, handle)) =
                spawn_server(config, Arc::new(StubKernel)).await
            else {
                return;
            };
            wait_for_tcp(addr).await;

            let endpoint = format!("http://{addr}");
            let mut client =
                pb::if97::if97_service_client::If97ServiceClient::connect(
                    endpoint,
                )
                .await
                .expect("grpc connect");

            let req = pb::if97::PtBatchRequest {
                batch_id: 42,
                p: vec![1.0, 2.0],
                t: vec![10.0, 20.0],
            };
            let mut resp = client
                .calculate_pt(tokio_stream::iter(vec![req]))
                .await
                .expect("calculate_pt")
                .into_inner();

            let msg = resp.message().await.expect("stream recv").expect("msg");
            assert_eq!(msg.batch_id, 42);
            assert_eq!(msg.batch_error_code, 0);
            assert!(msg.batch_error_message.is_empty());
            assert_eq!(msg.h, vec![11.0, 22.0]);
            assert_eq!(msg.status, vec![0, 0]);

            let _ = shutdown_tx.send(());
            let _ = tokio::time::timeout(Duration::from_secs(2), handle)
                .await
                .expect("server shutdown timeout");
        });
    }

    #[test]
    fn grpc_calculate_pt_length_mismatch() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .expect("tokio runtime");

        rt.block_on(async move {
            let config = test_config(1024);
            let Some((addr, shutdown_tx, handle)) =
                spawn_server(config, Arc::new(StubKernel)).await
            else {
                return;
            };
            wait_for_tcp(addr).await;

            let endpoint = format!("http://{addr}");
            let mut client =
                pb::if97::if97_service_client::If97ServiceClient::connect(
                    endpoint,
                )
                .await
                .expect("grpc connect");

            let req = pb::if97::PtBatchRequest {
                batch_id: 7,
                p: vec![1.0],
                t: vec![],
            };
            let mut resp = client
                .calculate_pt(tokio_stream::iter(vec![req]))
                .await
                .expect("calculate_pt")
                .into_inner();

            let msg = resp.message().await.expect("stream recv").expect("msg");
            assert_eq!(msg.batch_id, 7);
            assert_eq!(msg.batch_error_code, 1);
            assert!(msg.batch_error_message.contains("length mismatch"));
            assert!(msg.h.is_empty());
            assert!(msg.status.is_empty());

            let _ = shutdown_tx.send(());
            let _ = tokio::time::timeout(Duration::from_secs(2), handle)
                .await
                .expect("server shutdown timeout");
        });
    }

    #[test]
    fn grpc_calculate_pt_batch_too_large() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .expect("tokio runtime");

        rt.block_on(async move {
            let config = test_config(3);
            let Some((addr, shutdown_tx, handle)) =
                spawn_server(config, Arc::new(StubKernel)).await
            else {
                return;
            };
            wait_for_tcp(addr).await;

            let endpoint = format!("http://{addr}");
            let mut client =
                pb::if97::if97_service_client::If97ServiceClient::connect(
                    endpoint,
                )
                .await
                .expect("grpc connect");

            let req = pb::if97::PtBatchRequest {
                batch_id: 9,
                p: vec![1.0, 2.0, 3.0, 4.0],
                t: vec![10.0, 20.0, 30.0, 40.0],
            };
            let mut resp = client
                .calculate_pt(tokio_stream::iter(vec![req]))
                .await
                .expect("calculate_pt")
                .into_inner();

            let msg = resp.message().await.expect("stream recv").expect("msg");
            assert_eq!(msg.batch_id, 9);
            assert_eq!(msg.batch_error_code, 2);
            assert!(msg.batch_error_message.contains("batch too large"));
            assert!(msg.h.is_empty());
            assert!(msg.status.is_empty());

            let _ = shutdown_tx.send(());
            let _ = tokio::time::timeout(Duration::from_secs(2), handle)
                .await
                .expect("server shutdown timeout");
        });
    }

    #[test]
    fn grpc_healthcheck_serving() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .expect("tokio runtime");

        rt.block_on(async move {
            let config = test_config(1024);
            let Some((addr, shutdown_tx, handle)) =
                spawn_server(config, Arc::new(StubKernel)).await
            else {
                return;
            };
            wait_for_tcp(addr).await;

            let endpoint = format!("http://{addr}");
            let channel = tonic::transport::Channel::from_shared(endpoint)
                .expect("endpoint")
                .connect()
                .await
                .expect("grpc connect");
            let mut client =
                tonic_health::pb::health_client::HealthClient::new(channel);

            let resp = client
                .check(tonic_health::pb::HealthCheckRequest {
                    service: "if97.v1.If97Service".to_string(),
                })
                .await
                .expect("health check")
                .into_inner();

            let status = tonic_health::pb::health_check_response::ServingStatus::try_from(resp.status)
                .unwrap_or(tonic_health::pb::health_check_response::ServingStatus::Unknown);
            assert_eq!(
                status,
                tonic_health::pb::health_check_response::ServingStatus::Serving
            );

            let _ = shutdown_tx.send(());
            let _ = tokio::time::timeout(Duration::from_secs(2), handle)
                .await
                .expect("server shutdown timeout");
        });
    }

    struct BlockingKernel {
        entered_tx: std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
        gate: Arc<(std::sync::Mutex<bool>, std::sync::Condvar)>,
    }

    impl crate::kernel::Kernel for BlockingKernel {
        fn h_pt_batch(&self, p: &[f64], t: &[f64]) -> (Vec<f64>, Vec<u32>) {
            if let Ok(mut slot) = self.entered_tx.lock() {
                if let Some(tx) = slot.take() {
                    let _ = tx.send(());
                }
            }

            let (lock, cv) = &*self.gate;
            let mut allow = lock.lock().expect("gate lock");
            while !*allow {
                allow = cv.wait(allow).expect("gate wait");
            }

            (vec![p[0] + t[0]], vec![0])
        }
    }

    #[test]
    fn dispatcher_global_in_flight_blocks_second_dispatch() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .expect("tokio runtime");

        rt.block_on(async move {
            let (entered_tx, entered_rx) = tokio::sync::oneshot::channel();
            let gate = Arc::new((std::sync::Mutex::new(false), std::sync::Condvar::new()));

            let kernel = Arc::new(BlockingKernel {
                entered_tx: std::sync::Mutex::new(Some(entered_tx)),
                gate: gate.clone(),
            });

            // max_in_flight_global=1 => второй dispatch_pt должен ждать, пока первый
            // не освободит permit (то есть пока kernel не закончит работу).
            let dispatcher = crate::engine::Dispatcher::new(kernel, 1);

            let (tx, _rx) = tokio::sync::mpsc::channel::<
                Result<pb::if97::PtBatchResponse, tonic::Status>,
            >(8);

            let conn_sem = Arc::new(tokio::sync::Semaphore::new(2));
            let conn_permit_1 = conn_sem
                .clone()
                .acquire_owned()
                .await
                .expect("conn permit 1");
            let conn_permit_2 = conn_sem
                .clone()
                .acquire_owned()
                .await
                .expect("conn permit 2");

            dispatcher
                .dispatch_pt(
                    pb::if97::PtBatchRequest {
                        batch_id: 1,
                        p: vec![1.0],
                        t: vec![10.0],
                    },
                    tx.clone(),
                    conn_permit_1,
                )
                .await;

            // Ждем пока первый batch реально зайдет в kernel и заблокируется на gate.
            let _ = tokio::time::timeout(Duration::from_secs(1), entered_rx)
                .await
                .expect("kernel did not start");

            let dispatcher_2 = dispatcher.clone();
            let mut handle_2 = tokio::spawn(async move {
                dispatcher_2
                    .dispatch_pt(
                        pb::if97::PtBatchRequest {
                            batch_id: 2,
                            p: vec![2.0],
                            t: vec![20.0],
                        },
                        tx.clone(),
                        conn_permit_2,
                    )
                    .await;
            });

            // Пока gate закрыт, второй dispatch_pt должен быть заблокирован на глобальном semaphore.
            assert!(
                tokio::time::timeout(Duration::from_millis(50), &mut handle_2)
                    .await
                    .is_err()
            );

            // Открываем gate: первый batch завершается, освобождает permit => второй dispatch_pt должен пройти.
            {
                let (lock, cv) = &*gate;
                let mut allow = lock.lock().expect("gate lock");
                *allow = true;
                cv.notify_all();
            }

            tokio::time::timeout(Duration::from_secs(1), &mut handle_2)
                .await
                .expect("second dispatch did not unblock")
                .expect("second dispatch task");
        });
    }
}
