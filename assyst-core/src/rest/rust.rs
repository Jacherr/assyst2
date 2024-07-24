use reqwest::{Client, Error};
use serde::Deserialize;
use serde_json::json;
use std::borrow::{Borrow, Cow};

const API_BASE: &str = "https://play.rust-lang.org";
const GODBOLT: &str = "https://godbolt.org/api/compiler/nightly/compile";
const BENCHMARK_TEMPLATE: &str = r#"
#![feature(test)]
#[cfg(test)]
extern crate test;

use std::process::Command;

fn main() {
    let cmd = Command::new("cargo").arg("bench").output().unwrap();
    let cmd = String::from_utf8_lossy(if cmd.stdout.len() == 0 { &cmd.stderr } else { &cmd.stdout });
    
    println!("{}", cmd);
}

#[cfg(test)]
mod tests {
    {{code}}
}
"#;

pub enum OptimizationLevel {
    Debug,
    Release,
}

impl OptimizationLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            OptimizationLevel::Debug => "debug",
            OptimizationLevel::Release => "release",
        }
    }
}

#[derive(Deserialize)]
pub struct GodboltAsmBlock {
    pub text: String,
}

#[derive(Deserialize)]
pub struct GodboltResult {
    asm: Vec<GodboltAsmBlock>,
}

#[derive(Deserialize)]
pub struct ApiResult {
    pub stdout: String,
    pub stderr: String,
}
impl ApiResult {
    pub fn format(&self) -> &str {
        if self.stdout.is_empty() {
            &self.stderr
        } else {
            &self.stdout
        }
    }
}

pub async fn godbolt(client: &Client, code: &str) -> Result<String, Error> {
    Ok(client
        .post(GODBOLT)
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .json(&json!({
            "source": code,
            "compiler": "nightly",
            "options": {
                "userArguments": "-Copt-level=3 -C target-feature=+sse3,+avx -C target-cpu=native"
            },
            "lang": "rust",
            "allowStoreCodeDebug": true
        }))
        .send()
        .await?
        .json::<GodboltResult>()
        .await?
        .asm
        .into_iter()
        .map(|x| x.text)
        .collect::<Vec<_>>()
        .join("\n"))
}

#[allow(clippy::too_many_arguments)]
pub async fn request(
    client: &Client,
    path: &str,
    code: &str,
    channel: Option<&str>,
    mode: Option<&str>,
    edition: Option<&str>,
    crate_type: Option<&str>,
    tests: Option<bool>,
) -> Result<ApiResult, Error> {
    client
        .post(&format!("{API_BASE}/{path}"))
        .json(&json!({
            "code": code,
            "channel": channel.unwrap_or("stable"),
            "mode": mode.unwrap_or("debug"),
            "edition": edition.unwrap_or("2018"),
            "crateType": crate_type.unwrap_or("bin"),
            "tests": tests.unwrap_or(false)
        }))
        .send()
        .await?
        .json()
        .await
}

pub async fn run(
    client: &Client,
    code: &str,
    channel: Option<&str>,
    mode: Option<&str>,
    edition: Option<&str>,
    crate_type: Option<&str>,
    tests: Option<bool>,
) -> Result<ApiResult, Error> {
    request(client, "execute", code, channel, mode, edition, crate_type, tests).await
}

pub async fn miri(
    client: &Client,
    code: &str,
    channel: Option<&str>,
    opt: OptimizationLevel,
) -> Result<ApiResult, Error> {
    request(client, "miri", code, channel, Some(opt.as_str()), None, None, None).await
}

pub async fn clippy(
    client: &Client,
    code: &str,
    channel: Option<&str>,
    opt: OptimizationLevel,
) -> Result<ApiResult, Error> {
    request(client, "clippy", code, channel, Some(opt.as_str()), None, None, None).await
}

pub fn prepend_code(code: &str) -> Cow<str> {
    if !code.contains("fn main") {
        Cow::Owned(format!("fn main() {{ println!(\"{{:?}}\", {{ {code} }});}}"))
    } else {
        Cow::Borrowed(code)
    }
}

pub async fn run_miri(client: &Client, code: &str, channel: &str, opt: OptimizationLevel) -> Result<ApiResult, Error> {
    let code = prepend_code(code);

    miri(client, &code, Some(channel), opt).await
}

pub async fn run_clippy(
    client: &Client,
    code: &str,
    channel: &str,
    opt: OptimizationLevel,
) -> Result<ApiResult, Error> {
    let code = prepend_code(code);

    clippy(client, &code, Some(channel), opt).await
}

pub async fn run_binary(
    client: &Client,
    code: &str,
    channel: &str,
    opt: OptimizationLevel,
) -> Result<ApiResult, Error> {
    let code = prepend_code(code);

    run(
        client,
        code.borrow(),
        Some(channel),
        Some(opt.as_str()),
        None,
        None,
        None,
    )
    .await
}

pub async fn run_benchmark(client: &Client, code: &str) -> Result<ApiResult, Error> {
    let code = BENCHMARK_TEMPLATE.replace("{{code}}", code);

    run_binary(client, &code, "nightly", OptimizationLevel::Release).await
}

pub async fn run_godbolt(client: &Client, code: &str) -> Result<ApiResult, Error> {
    let asm = godbolt(client, code).await?;

    Ok(ApiResult {
        stdout: asm,
        stderr: String::new(),
    })
}
