use crate::rest::patreon::Patron;
use anyhow::bail;
use assyst_common::config::CONFIG;
use assyst_common::err;
use assyst_database::model::free_tier_2_requests::FreeTier2Requests;
use assyst_database::DatabaseHandler;
use bincode::{deserialize, serialize};
use shared::errors::ProcessingError;
use shared::fifo::{FifoSend, WsiRequest};
use shared::job::JobResult;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::{self, Sender};
use tokio::time::sleep;
use tracing::info;

pub mod jobs;

static CONNECTED: AtomicBool = AtomicBool::new(false);
pub type WsiSender = (Sender<JobResult>, FifoSend, usize);

pub struct WsiHandler {
    database_handler: Arc<DatabaseHandler>,
    premium_users: Arc<Mutex<Vec<Patron>>>,
    pub wsi_tx: UnboundedSender<WsiSender>,
}
impl WsiHandler {
    pub fn new(database_handler: Arc<DatabaseHandler>, premium_users: Arc<Mutex<Vec<Patron>>>) -> WsiHandler {
        let (tx, rx) = unbounded_channel::<WsiSender>();
        Self::listen(rx, &CONFIG.urls.wsi);
        WsiHandler {
            wsi_tx: tx,
            database_handler,
            premium_users,
        }
    }

    pub fn listen(job_rx: UnboundedReceiver<WsiSender>, socket: &str) {
        let job_rx = Arc::new(tokio::sync::Mutex::new(job_rx));
        let socket = socket.to_owned();

        spawn(async move {
            let mut retries = 0;
            loop {
                let stream = match TcpStream::connect(&socket).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        if retries > CONFIG.dev.wsi_retry_limit {
                            break;
                        }
                        err!(
                            "Failed to connect to WSI server ({}), attempting reconnection in 10 sec...",
                            e.to_string()
                        );
                        sleep(Duration::from_secs(10)).await;
                        retries += 1;
                        continue;
                    },
                };

                info!("Connected to WSI");

                CONNECTED.store(true, Ordering::Relaxed);

                let (mut reader, mut writer) = stream.into_split();

                let jobs = Arc::new(Mutex::new(HashMap::<usize, Sender<JobResult>>::new()));
                let jobs_clone = jobs.clone();
                let jobs_clone_2 = jobs.clone();

                let mut r = tokio::spawn(async move {
                    loop {
                        let length = match reader.read_u32().await {
                            Err(e) => {
                                err!("Failed to read length from WSI: {:?}", e);
                                break;
                            },
                            Ok(x) => x,
                        };
                        let mut buf = vec![0; length as usize];
                        if let Err(e) = reader.read_exact(&mut buf).await {
                            err!("Failed to read buffer from WSI: {:?}", e);
                            break;
                        }

                        let deserialized = match deserialize::<JobResult>(&buf) {
                            Ok(x) => x,
                            Err(e) => {
                                err!("Failed to deserialize WSI data: {:?}", e);
                                continue;
                            },
                        };

                        let job_id = deserialized.id();
                        let tx = jobs_clone.lock().unwrap().remove(&job_id);

                        if let Some(tx) = tx {
                            // if this fails it means it timed out
                            let res = tx.send(deserialized);
                            if res.is_err() {
                                err!("Failed to send job result ID {} to job sender", job_id);
                            }
                        }
                    }
                });

                let job_rx_clone = job_rx.clone();

                let mut w = tokio::spawn(async move {
                    let next_job_id = AtomicUsize::new(0);

                    loop {
                        let (tx, job, premium_level) = match job_rx_clone.lock().await.recv().await {
                            Some(x) => x,
                            None => break,
                        };

                        let id = next_job_id.fetch_add(1, Ordering::Relaxed);

                        let wsi_request = WsiRequest::new(id, premium_level, job);
                        let job = serialize(&wsi_request).unwrap();

                        jobs.lock().unwrap().insert(id, tx);

                        if let Err(e) = writer.write_u32(job.len() as u32).await {
                            err!("Failed to write to WSI: {:?}", e.to_string());
                            break;
                        }

                        if let Err(e) = writer.write_all(&job).await {
                            err!("Failed to write to WSI: {:?}", e.to_string());
                            break;
                        }
                    }
                });

                let _ = tokio::select! {
                    output = &mut r => { w.abort(); output },
                    output = &mut w => { r.abort(); output },
                };

                CONNECTED.store(false, Ordering::Relaxed);
                {
                    let mut lock = jobs_clone_2.lock().unwrap();
                    let keys = lock.keys().copied().collect::<Vec<_>>().clone();
                    for job in keys {
                        let tx = lock.remove(&job).unwrap();
                        let _ = tx.send(JobResult::new_err(
                            0,
                            ProcessingError::Other("The image server died. Try again in a few seconds.".to_owned()),
                        ));
                    }
                };

                err!("Lost connection to WSI server, attempting reconnection in 10 sec...");
                sleep(Duration::from_secs(10)).await;
            }
        });
    }

    /// This function will remove a free voter request if the user has any
    /// and are not a patron!
    pub async fn get_request_tier(&self, user_id: u64) -> Result<usize, anyhow::Error> {
        if let Some(p) = {
            let premium_users = self.premium_users.lock().unwrap();
            premium_users.iter().find(|i| i.user_id == user_id).cloned()
        } {
            return Ok(p.tier as usize);
        }

        let user_tier2 = FreeTier2Requests::get_user_free_tier_2_requests(&*self.database_handler, user_id).await?;

        if user_tier2.count > 0 {
            user_tier2
                .change_free_tier_2_requests(&*self.database_handler, -1)
                .await?;
            Ok(2)
        } else {
            Ok(0)
        }
    }

    pub async fn run_job(&self, job: FifoSend, user_id: u64) -> anyhow::Result<Vec<u8>> {
        if !CONNECTED.load(Ordering::Relaxed) {
            bail!("Assyst cannot establish a connection to the image server at this time. Try again in a few minutes.");
        }

        let premium_level = self.get_request_tier(user_id).await?;

        let (tx, rx) = oneshot::channel::<JobResult>();
        self.wsi_tx.send((tx, job, premium_level))?;

        let res = rx.await;
        let result = res.unwrap_or(JobResult::new_err(
            0,
            ProcessingError::Other("The image server died. Try again in a couple of minutes.".to_string()),
        ));

        Ok(result.result?)
    }
}
