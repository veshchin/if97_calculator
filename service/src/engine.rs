use crate::kernel::Kernel;
use crate::pb;
use std::sync::Arc;
use tokio::sync::{mpsc, OwnedSemaphorePermit, Semaphore};
use tonic::Status;

#[derive(Clone)]
pub struct Dispatcher {
    kernel: Arc<dyn Kernel>,
    global: Arc<Semaphore>,
}

impl Dispatcher {
    pub fn new(kernel: Arc<dyn Kernel>, max_in_flight_global: usize) -> Self {
        Self {
            kernel,
            global: Arc::new(Semaphore::new(max_in_flight_global.max(1))),
        }
    }

    pub async fn dispatch_pt(
        &self,
        batch: pb::if97::PtBatchRequest,
        tx: mpsc::Sender<Result<pb::if97::PtBatchResponse, Status>>,
        conn_permit: OwnedSemaphorePermit,
    ) {
        let global_permit = match self.global.clone().acquire_owned().await {
            Ok(permit) => permit,
            Err(_) => return,
        };

        let kernel = self.kernel.clone();
        rayon::spawn(move || {
            let batch_id = batch.batch_id;
            let p = batch.p;
            let t = batch.t;
            let (h, status) = kernel.h_pt_batch(&p, &t);

            let _ = tx.blocking_send(Ok(pb::if97::PtBatchResponse {
                batch_id,
                h,
                status,
                batch_error_code: 0,
                batch_error_message: String::new(),
            }));

            drop(global_permit);
            drop(conn_permit);
        });
    }
}

