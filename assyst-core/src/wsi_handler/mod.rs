use crate::assyst::ThreadSafeAssyst;
use anyhow::bail;
use assyst_common::config::CONFIG;
use assyst_common::err;
use assyst_database::model::free_tier_2_requests::FreeTier2Requests;
use bincode::{deserialize, serialize};
use shared::errors::ProcessingError;
use shared::fifo::{FifoSend, WsiRequest};
use shared::job::JobResult;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::{self, Sender};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::info;

pub mod jobs;

static CONNECTED: AtomicBool = AtomicBool::new(false);
pub type WsiSender = (Sender<JobResult>, FifoSend, usize);

pub struct WsiHandler {
    pub wsi_tx: UnboundedSender<WsiSender>,
}
impl WsiHandler {
    pub fn new() -> WsiHandler {
        let (tx, rx) = unbounded_channel::<WsiSender>();
        Self::listen(rx, &CONFIG.urls.wsi);
        WsiHandler { wsi_tx: tx }
    }

    pub fn listen(job_rx: UnboundedReceiver<WsiSender>, socket: &str) {
        let job_rx = Arc::new(Mutex::new(job_rx));
        let socket = socket.to_owned();

        spawn(async move {
            loop {
                let stream = match TcpStream::connect(&socket).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        err!(
                            "Failed to connect to WSI server ({}), attempting reconnection in 10 sec...",
                            e.to_string()
                        );
                        sleep(Duration::from_secs(10)).await;
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
                        match reader.read_exact(&mut buf).await {
                            Err(e) => {
                                err!("Failed to read buffer from WSI: {:?}", e);
                                break;
                            },
                            _ => {},
                        }

                        let deserialized = match deserialize::<JobResult>(&buf) {
                            Ok(x) => x,
                            Err(e) => {
                                err!("Failed to deserialize WSI data: {:?}", e);
                                continue;
                            },
                        };

                        let job_id = deserialized.id();
                        let tx = jobs_clone.lock().await.remove(&job_id);

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

                        jobs.lock().await.insert(id, tx);

                        match writer.write_u32(job.len() as u32).await {
                            Err(e) => {
                                err!("Failed to write to WSI: {:?}", e.to_string());
                                break;
                            },
                            _ => {},
                        }

                        match writer.write_all(&job).await {
                            Err(e) => {
                                err!("Failed to write to WSI: {:?}", e.to_string());
                                break;
                            },
                            _ => {},
                        }
                    }
                });

                let _ = tokio::select! {
                    output = &mut r => { w.abort(); output },
                    output = &mut w => { r.abort(); output },
                };

                CONNECTED.store(false, Ordering::Relaxed);
                let mut lock = jobs_clone_2.lock().await;
                let keys = lock.keys().map(|x| *x).collect::<Vec<_>>().clone();

                for job in keys {
                    let tx = lock.remove(&job).unwrap();
                    let _ = tx.send(JobResult::new_err(
                        0,
                        ProcessingError::Other("The image server died. Try again in a few seconds.".to_owned()),
                    ));
                }

                err!("Lost connection to WSI server, attempting reconnection in 10 sec...");
                sleep(Duration::from_secs(10)).await;
            }
        });
    }

    /// This function will remove a free voter request if the user has any
    /// and are not a patron!
    pub async fn get_request_tier(assyst: ThreadSafeAssyst, user_id: u64) -> Result<usize, anyhow::Error> {
        let patrons = assyst.patrons.lock().unwrap();
        let patron = patrons.iter().find(|i| i.user_id == user_id);
        if let Some(p) = patron {
            return Ok(p.tier.clone() as usize);
        }

        let user_tier1 =
            FreeTier2Requests::get_user_free_tier_2_requests(&*assyst.database_handler.read().await, user_id).await?;

        if user_tier1.count > 0 {
            user_tier1
                .change_free_tier_2_requests(&*assyst.database_handler.read().await, -1)
                .await?;
            Ok(2)
        } else {
            Ok(0)
        }
    }

    pub async fn run_job(&self, assyst: ThreadSafeAssyst, job: FifoSend, user_id: u64) -> anyhow::Result<Vec<u8>> {
        if !CONNECTED.load(Ordering::Relaxed) {
            bail!("Assyst cannot establish a connection to the image server at this time. Try again in a few minutes.");
        }

        let premium_level = Self::get_request_tier(assyst.clone(), user_id).await?;

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
