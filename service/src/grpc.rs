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
