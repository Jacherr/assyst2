pub mod rate_tracker;

/// Attempts to extract memory usage
pub fn get_memory_usage() -> Option<usize> {
    let field = 1;
    let contents = std::fs::read("/proc/self/statm").ok()?;
    let contents = String::from_utf8(contents).ok()?;
    let s = contents.split_whitespace().nth(field)?;
    let npages = s.parse::<usize>().ok()?;
    Some(npages * 4096)
}