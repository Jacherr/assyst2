use rayon::prelude::*;
use std::process::Command;

use crate::ansi::Ansi;

static PROCESSES: &[&'static str] = &["assyst-core", "assyst-cache", "assyst-gateway", "filer"];
static HOST_PROCESS: &'static str = "host machine";

/// Attempts to extract memory usage in bytes for a process by PID
pub fn get_memory_usage_for(pid: &str) -> Option<usize> {
    let field = 1;
    let contents = std::fs::read(&format!("/proc/{pid}/statm")).ok()?;
    let contents = String::from_utf8(contents).ok()?;
    let s = contents.split_whitespace().nth(field)?;
    let npages = s.parse::<usize>().ok()?;
    Some(npages * 4096)
}

pub fn get_host_memory_usage() -> Option<usize> {
    let output = exec_sync("free -b | head -2 | tail -1 | awk {{'print $3'}}");
    output.ok().map(|x| x.stdout.trim().parse::<usize>().ok()).flatten()
}

/// Gets the memory usage in bytes of all 'relevant'' processes.
pub fn get_processes_mem_usage() -> Vec<(&'static str, usize)> {
    let mut memory_usages: Vec<(&str, usize)> = vec![];

    for process in PROCESSES {
        let pid = pid_of(process).unwrap_or(0).to_string();
        let mem_usage = get_memory_usage_for(&pid).unwrap_or(0);
        memory_usages.push((process, mem_usage));
    }

    memory_usages.push((HOST_PROCESS, get_host_memory_usage().unwrap_or(0)));

    memory_usages
}

/// Attempts to get CPU usage for a PID
pub fn cpu_usage_percentage_of(pid: usize) -> Option<f64> {
    let output = exec_sync(&format!("top -b -n 2 -d 0.2 -p {pid} | tail -1 | awk '{{print $9}}'"));
    output
        .ok()
        .map(|x| x.stdout.trim().parse::<f64>().ok())
        .flatten()
        .map(|x| x / num_cpus::get() as f64)
}

/// Gets the CPU usage of the host machine
pub fn get_host_cpu_usage() -> Option<f64> {
    let output = exec_sync("top -bn2 -d 0.3 | grep '%Cpu' | tail -1");

    output
        .ok()
        .map(|x| x.stdout.trim().split(",").nth(3).map(|y| y.to_owned()))
        .flatten()
        .map(|x| x.trim().split(" ").nth(0).map(|y| y.trim().to_owned()))
        .flatten()
        .map(|x| x.trim().parse::<f64>().ok().map(|x| 100.0 - x))
        .flatten()
}

/// Gets the CPU usage of all 'relevant' processes
pub fn get_processes_cpu_usage() -> Vec<(&'static str, f64)> {
    let mut cpu_usages: Vec<(&str, f64)> = PROCESSES.iter().map(|x| (*x, 0.0)).collect::<Vec<_>>();
    cpu_usages.push((HOST_PROCESS, 0.0));

    cpu_usages.par_iter_mut().for_each(|entry| {
        if entry.0 == HOST_PROCESS {
            entry.1 = get_host_cpu_usage().unwrap_or(0.0);
        } else {
            let pid = pid_of(entry.0).unwrap_or(0);
            let cpu_usage = cpu_usage_percentage_of(pid).unwrap_or(0.0);
            entry.1 = cpu_usage;
        }
    });

    cpu_usages
}

/// Gets the uptime of a process based on its PID
pub fn get_uptime_of(pid: usize) -> Option<String> {
    exec_sync(&format!("ps -p {pid} -o etime="))
        .ok()
        .map(|x| x.stdout.trim().to_owned())
}

/// Gets the uptimes of all 'relevant' processes
pub fn get_processes_uptimes() -> Vec<(&'static str, String)> {
    let mut uptimes: Vec<(&str, String)> = vec![];

    for process in PROCESSES {
        let pid = pid_of(process).unwrap_or(0);
        let uptime = get_uptime_of(pid).unwrap_or("unknown".to_owned());
        let uptime = if uptime.is_empty() {
            "offline".fg_red().to_owned()
        } else {
            uptime
        };
        uptimes.push((process, uptime));
    }

    let host_uptime = get_uptime_of(1).unwrap_or("unknown".to_owned());
    uptimes.push((HOST_PROCESS, host_uptime));

    uptimes
}

/// Attempts to get the PID of a process by its name
pub fn pid_of(name: &str) -> Option<usize> {
    let result = exec_sync(&format!("pidof {name}")).ok()?.stdout;
    Some(result.trim().parse().ok()?)
}

#[derive(Clone, Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
}

/// Executes a bash command
pub fn exec_sync(command: &str) -> Result<CommandOutput, std::io::Error> {
    let mut cmd = Command::new("bash");
    cmd.args(&["-c", command]);

    let output = cmd.output()?;

    Ok(CommandOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}
