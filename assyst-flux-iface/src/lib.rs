#![feature(let_chains)]

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context};
use assyst_common::config::CONFIG;
use assyst_common::util::process::exec_sync;
use assyst_common::util::{hash_buffer, string_from_likely_utf8};
use assyst_database::model::free_tier_2_requests::FreeTier2Requests;
use assyst_database::DatabaseHandler;
use flux_request::{FluxRequest, FluxStep};
use jobs::FluxResult;
use libc::pid_t;
use tokio::fs;
use tokio::process::Command;
use tokio::time::timeout;

pub mod flux_request;
pub mod jobs;
pub mod limits;

const FLUX_PATH: &str = "./target/release/flux";
const FLUX_DIR: &str = "./flux";
const LD_LIBRARY_PATH: &str = "./build";

struct FileDeletionDefer(String);
impl Drop for FileDeletionDefer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0.clone());
    }
}

static COMPILING: AtomicBool = AtomicBool::new(false);
struct CompilingCompleteDefer {}
impl Drop for CompilingCompleteDefer {
    fn drop(&mut self) {
        COMPILING.fetch_and(false, Ordering::Relaxed);
    }
}

pub struct FluxHandler {
    database_handler: Arc<DatabaseHandler>,
    premium_users: Arc<Mutex<HashMap<u64, u64>>>,
}
impl FluxHandler {
    pub fn new(database_handler: Arc<DatabaseHandler>, premium_users: Arc<Mutex<HashMap<u64, u64>>>) -> Self {
        Self {
            database_handler,
            premium_users,
        }
    }

    pub fn set_premium_users(&self, users: HashMap<u64, u64>) {
        *self.premium_users.lock().unwrap() = users;
    }

    pub async fn run_flux(&self, request: FluxRequest, time_limit: Duration) -> FluxResult {
        let mut input_file_paths: Vec<String> = vec![];
        let mut output_file_path: String = String::new();
        let mut args: Vec<String> = vec![];

        for step in request.0 {
            match step {
                FluxStep::Input(i) => {
                    let hash = hash_buffer(&i);
                    let path = format!("/tmp/{hash}");
                    fs::write(&path, &i).await?;
                    input_file_paths.push(path.clone());

                    args.push("-i".to_owned());
                    args.push(path);
                },
                FluxStep::Operation((operation, options)) => {
                    let mut op_full = operation;

                    if !options.is_empty() {
                        op_full += "[";
                        for op in options.iter() {
                            op_full += op.0;
                            op_full += "=";
                            op_full += &op.1.replace(";", "\\;");
                            op_full += ";";
                        }
                        // remove trailing ";"
                        op_full = op_full[..op_full.len() - 1].to_owned();

                        op_full += "]";
                    }

                    args.push("-o".to_owned());
                    args.push(op_full);
                },
                FluxStep::Output => {
                    let unix = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("time went backwards")
                        .as_millis();

                    let path = format!("/tmp/{unix}");
                    args.push(path.clone());
                    output_file_path = path;
                },
                FluxStep::ImagePageLimit(l) => {
                    args.push("--page-limit".to_owned());
                    args.push(l.to_string())
                },
                FluxStep::ResolutionLimit((w, h)) => {
                    args.push("--res-limit".to_owned());
                    args.push(format!("{w}x{h}"));
                },
                FluxStep::VideoDecodeDisabled => {
                    args.push("--disable-video-decode".to_owned());
                },
                FluxStep::Info => {
                    args.push("--info".to_owned());
                },
                FluxStep::Version => args.push("--version".to_owned()),
            }
        }

        // defer file deletions to func return
        let mut defers: Vec<FileDeletionDefer> = vec![];
        for input in input_file_paths {
            defers.push(FileDeletionDefer(input.clone()));
        }
        defers.push(FileDeletionDefer(output_file_path.clone()));

        let flux_workspace_root = if CONFIG.dev.flux_workspace_root_path_override.is_empty() {
            FLUX_DIR.to_owned()
        } else {
            CONFIG.dev.flux_workspace_root_path_override.clone()
        };

        if COMPILING.load(Ordering::Relaxed) {
            bail!("The image service is still preparing. Try again in a few seconds.");
        }

        let mut command = Command::new(FLUX_PATH);
        command.args(args);
        command.current_dir(flux_workspace_root);
        command.env("LD_LIBRARY_PATH", LD_LIBRARY_PATH);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let spawn = command.spawn().context("Failed to execute flux")?;
        let id = spawn.id();
        let output = timeout(time_limit, spawn.wait_with_output()).await;

        let output = (match output {
            Ok(o) => o,
            Err(_) => {
                // send SIGTERM to flux to clean up child processes
                if let Some(id) = id {
                    unsafe { libc::kill(id as pid_t, libc::SIGTERM) };
                };
                bail!("The operation timed out");
            },
        })
        .context("Failed to execute flux")?;

        if !output.status.success() {
            bail!(
                "Something went wrong ({}): {}",
                output.status.to_string(),
                string_from_likely_utf8(output.stderr).trim()
            );
        }

        let output = if !output_file_path.is_empty() {
            fs::read(&output_file_path)
                .await
                .context("Failed to read output file")?
        } else {
            output.stdout
        };

        Ok(output)
    }

    pub async fn compile_flux() -> anyhow::Result<()> {
        const CARGO_EXIT_FAIL: i32 = 101;
        COMPILING.fetch_or(true, Ordering::Relaxed);
        let _defer = CompilingCompleteDefer {};

        let res = exec_sync(&format!(
            "cd {} && rm {FLUX_PATH} && mold -run ~/.cargo/bin/cargo build -q --release",
            if CONFIG.dev.flux_workspace_root_path_override.is_empty() {
                FLUX_DIR
            } else {
                &CONFIG.dev.flux_workspace_root_path_override
            }
        ))
        .context("Failed to compile flux")?;

        if res.exit_code.code() == Some(CARGO_EXIT_FAIL) {
            bail!("{}", res.stderr);
        }

        Ok(())
    }

    /// This function will remove a free voter request if the user has any
    /// and are not a patron!
    pub async fn get_request_tier(&self, user_id: u64) -> Result<usize, anyhow::Error> {
        if let Some(p) = {
            let premium_users = self.premium_users.lock().unwrap();
            premium_users.get(&user_id).copied()
        } {
            return Ok((p as usize).saturating_sub(1));
        }

        let user_tier2 = FreeTier2Requests::get_user_free_tier_2_requests(&self.database_handler, user_id).await?;

        if user_tier2.count > 0 {
            user_tier2
                .change_free_tier_2_requests(&self.database_handler, -1)
                .await?;
            Ok(2)
        } else {
            Ok(0)
        }
    }

    pub async fn get_version(&self) -> anyhow::Result<String> {
        let mut req = FluxRequest::default();
        req.version();
        Ok(string_from_likely_utf8(self.run_flux(req, Duration::MAX).await?))
    }
}
