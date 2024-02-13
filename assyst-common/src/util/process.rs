use std::process::Command;

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

/// Attempts to get the PID of a process by its name
pub fn pid_of(name: &str) -> Option<usize> {
    let result = exec_sync(&format!("pidof {name}")).ok()?.stdout;
    Some(result.trim().parse().ok()?)
}

/// Attempts to get CPU usage for a PID
pub fn cpu_usage_percentage_of(pid: usize) -> Option<f64> {
    let output = exec_sync(&format!("top -b -n 2 -d 0.2 -p {pid} | tail -1 | awk '{{print $9}}'"));
    output.ok().map(|x| x.stdout.parse::<f64>().ok()).flatten()
}

/// Gets the memory usage in bytes of all key Assyst processes.
pub fn get_processes_mem_usage() -> Vec<(&'static str, usize)> {
    let core_memory_usage = get_own_memory_usage().unwrap_or(0);

    let gateway_pid = pid_of("assyst-gateway").unwrap_or(0).to_string();
    let gateway_memory_usage = get_memory_usage_for(&gateway_pid).unwrap_or(0);

    let cache_pid = pid_of("assyst-cache").unwrap_or(0).to_string();
    let cache_memory_usage = get_memory_usage_for(&cache_pid).unwrap_or(0);

    let memory_usages = vec![
        ("assyst-core", core_memory_usage),
        ("assyst-gateway", gateway_memory_usage),
        ("assyst-cache", cache_memory_usage),
    ];

    memory_usages
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
