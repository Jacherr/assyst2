use std::process::Command;

/// Attempts to extract memory usage for a process by pid
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

/// Attempts to extract memory usage for the current process
pub fn pid_of(pid: &str) -> Option<usize> {
    let result = exec_sync(&format!("pidof {pid}")).ok()?.stdout;
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
