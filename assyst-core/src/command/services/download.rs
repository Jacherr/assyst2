use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use assyst_common::util::{filetype, format_duration, sanitise_filename};
use assyst_proc_macro::command;
use assyst_string_fmt::Markdown;
use rand::{thread_rng, Rng};
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tokio::time::{sleep, timeout};
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_util::builder::command::StringBuilder;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::command::arguments::{ParseArgument, Word};
use crate::command::errors::TagParseError;
use crate::command::flags::{flags_from_str, FlagDecode, FlagType};
use crate::command::messagebuilder::Attachment;
use crate::command::{Availability, Category, CommandCtxt, InteractionCommandParseCtxt, Label, RawMessageParseCtxt};
use crate::flag_parse_argument;
use crate::rest::web_media_download::{download_web_media, get_youtube_playlist_entries, WebDownloadOpts};

#[derive(Default)]
pub struct DownloadFlags {
    pub audio: bool,
    pub quality: u64,
    pub verbose: bool,
}
impl FlagDecode for DownloadFlags {
    fn from_str(input: &str) -> anyhow::Result<Self> {
        let mut valid_flags = HashMap::new();
        valid_flags.insert("quality", FlagType::WithValue);
        valid_flags.insert("audio", FlagType::NoValue);
        valid_flags.insert("verbose", FlagType::NoValue);

        let raw_decode = flags_from_str(input, valid_flags)?;
        let result = Self {
            audio: raw_decode.contains_key("audio"),
            quality: raw_decode
                .get("quality")
                .unwrap_or(&None)
                .clone()
                .unwrap_or("720".to_owned())
                .parse()
                .context("Provided quality is invalid")?,
            verbose: raw_decode.contains_key("verbose"),
        };

        Ok(result)
    }
}
flag_parse_argument! { DownloadFlags }

#[command(
    name = "download",
    aliases = ["dl"],
    description = "download media from a website",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Services,
    usage = "[url] <flags>",
    examples = ["https://youtu.be/dQw4w9WgXcQ", "https://youtu.be/dQw4w9WgXcQ --audio", "https://youtu.be/dQw4w9WgXcQ --quality 480"],
    send_processing = true,
    flag_descriptions = [
        ("audio", "Get content as MP3"),
        ("quality [quality:144|240|360|480|720|1080|max]", "Set resolution of output"),
    ]
)]
pub async fn download(ctxt: CommandCtxt<'_>, url: Word, options: DownloadFlags) -> anyhow::Result<()> {
    let mut opts =
        WebDownloadOpts::from_download_flags(options, ctxt.assyst().rest_cache_handler.get_web_download_urls());

    if url.0.to_ascii_lowercase().contains("youtube.com/playlist") {
        let videos = get_youtube_playlist_entries(&url.0).await?;

        let videos_len = videos.len();

        let videos = videos.iter().take(100).collect::<Vec<_>>();

        opts.audio_only = Some(true);

        let main_msg = format!(
            "{}Downloading {} videos as MP3. This may take a while!",
            if videos_len > 100 {
                format!(":warning: Playlist has {videos_len} videos, but the download limit is 100\n")
            } else {
                "".to_owned()
            },
            videos.len()
        );

        ctxt.reply(&main_msg[..]).await?;

        let len = videos.len();
        let mut video_tasks = JoinSet::new();

        let max_concurrent_downloads = num_cpus::get();

        let mut locks: Vec<Mutex<()>> = Vec::new();
        for _ in 0..max_concurrent_downloads {
            locks.push(Mutex::new(()));
        }
        let locks = Arc::new(locks);
        let zip = Arc::new(Mutex::new(ZipWriter::new(Cursor::new(Vec::new()))));
        let failed = Arc::new(std::sync::Mutex::new(Vec::new()));

        for v in videos {
            let a = ctxt.assyst().clone();
            let url = v.1.clone();
            let title = v.0.clone();
            let opts = opts.clone();
            let l = locks.clone();
            let z = zip.clone();
            let failed = failed.clone();
            video_tasks.spawn(async move {
                let _lock = loop {
                    let r#try = l.iter().find(|x| x.try_lock().is_ok());
                    match r#try {
                        Some(l) => break l.lock().await,
                        None => {
                            let time = thread_rng().gen_range(10..1500);
                            sleep(Duration::from_millis(time)).await;
                        },
                    }
                };

                let media = timeout(
                    Duration::from_secs(120),
                    download_web_media(&a.reqwest_client, &url, opts),
                )
                .await;
                match media {
                    Ok(Ok(m)) => {
                        let mut z_lock = z.lock().await;
                        let r#type = match filetype::get_sig(&m) {
                            Some(t) => t,
                            None => {
                                failed.lock().unwrap().push(format!("{url}: Unknown signature"));
                                return;
                            },
                        };

                        let _ = z_lock
                            .start_file(
                                format!(
                                    "{}_{}.{}",
                                    sanitise_filename(&title),
                                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
                                    r#type.as_str()
                                ),
                                SimpleFileOptions::default(),
                            )
                            .map_err(|e| {
                                failed
                                    .lock()
                                    .unwrap()
                                    .push(format!("{url}: failed to start file ({e:?})"))
                            });

                        let _ = z_lock.write_all(&m).map_err(|e| {
                            failed
                                .lock()
                                .unwrap()
                                .push(format!("{url}: failed to write file ({e:?})"))
                        });
                    },
                    Ok(Err(e)) => {
                        failed.lock().unwrap().push(format!("{url}: {e:?}"));
                    },
                    Err(_) => {
                        failed.lock().unwrap().push(format!("{url}: timed out"));
                    },
                }
            });
        }

        let mut count = 0;
        let mut joined = Vec::new();

        while let Some(v) = video_tasks.join_next().await {
            count += 1;
            if count % 5 == 0 {
                ctxt.reply(format!("{main_msg}\nDownloaded {count}/{len} videos."))
                    .await?;
            }
            joined.push(v?);
        }

        let finished = Arc::into_inner(zip)
            .context("`Arc` has more than one strong reference")?
            .into_inner();
        let finished = finished.finish().context("Failed to create ZIP")?;
        let out = finished.clone().into_inner();

        ctxt.reply(format!(
            "Uploading ZIP. The ZIP is {}, so uploading may take some time.",
            human_bytes::human_bytes(out.len() as f64)
        ))
        .await?;

        ctxt.reply((
            Attachment {
                name: "files.zip".to_owned().into_boxed_str(),
                data: out,
            },
            {
                let failed = failed.lock().unwrap();
                if !failed.is_empty() {
                    format!(
                        ":warning: Failed to download {} videos, most likely due to region or age restrictions.{}",
                        failed.len(),
                        if opts.verbose {
                            let j = failed.join("\n");
                            format!("\nThe errors were: {}", (&j[0..1500.min(j.len())]).codeblock(""))
                        } else {
                            "".to_owned()
                        }
                    )
                } else {
                    "No videos failed to download.".to_owned()
                }
            },
        ))
        .await?;
    } else {
        let result = download_web_media(&ctxt.assyst().reqwest_client, &url.0, opts).await?;

        ctxt.reply((
            result,
            &format!(
                "Took {}",
                format_duration(&ctxt.data.execution_timings.processing_time_start.elapsed())
            )[..],
        ))
        .await?;
    }

    Ok(())
}
