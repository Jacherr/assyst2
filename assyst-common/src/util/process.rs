use std::process::Command;

static PROCESSES: &[&'static str] = &["assyst-core", "assyst-cache", "assyst-gateway"];

/// Attempts to extract memory usage in bytes for a process by PID
pub fn get_memory_usage_for(pid: &str) -> Option<usize> {
    let field = 1;
    let contents = std::fs::read(&format!("/proc/{pid}/statm")).ok()?;
    let contents = String::from_utf8(contents).ok()?;
    let s = contents.split_whitespace().nth(field)?;
    let npages = s.parse::<usize>().ok()?;
    Some(npages * 4096)
}

/// Attempts to extract memory usage for the current process
pub fn get_own_memory_usage() -> Option<usize> {
    get_memory_usage_for("self")
}

/// Gets the memory usage in bytes of all key Assyst processes.
pub fn get_processes_mem_usage() -> Vec<(&'static str, usize)> {
    let mut memory_usages: Vec<(&str, usize)> = vec![];

    for process in PROCESSES {
        let pid = pid_of(process).unwrap_or(0).to_string();
        let mem_usage = get_memory_usage_for(&pid).unwrap_or(0);
        memory_usages.push((process, mem_usage));
    }

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

pub fn get_processes_cpu_usage() -> Vec<(&'static str, f64)> {
    let mut cpu_usages: Vec<(&str, f64)> = vec![];

    for process in PROCESSES {
        let pid = pid_of(process).unwrap_or(0);
        let cpu_usage = cpu_usage_percentage_of(pid).unwrap_or(0.0);
        cpu_usages.push((process, cpu_usage));
    }

    cpu_usages
}

pub fn get_uptime_of(pid: usize) -> Option<String> {
    exec_sync(&format!("ps -p {pid} -o etime="))
        .ok()
        .map(|x| x.stdout.trim().to_owned())
}

pub fn get_processes_uptimes() -> Vec<(&'static str, String)> {
    let mut uptimes: Vec<(&str, String)> = vec![];

    for process in PROCESSES {
        let pid = pid_of(process).unwrap_or(0);
        let uptime = get_uptime_of(pid).unwrap_or("unknown".to_string());
        uptimes.push((process, uptime));
    }

    uptimes
}

/// Attempts to get the PID of a process by its name
pub fn pid_of(name: &str) -> Option<usize> {
    let result = exec_sync(&format!("pidof {name}")).ok()?.stdout;
    Some(result.trim().parse().ok()?)
}

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
