/// Returns the longer string of the two given strings
fn get_longer_str<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}

/// Generates a table given a list of tuples containing strings
pub fn key_value(input: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    let longest: &str = input
        .iter()
        .map(|(x, y)| (x.as_ref(), y.as_ref()))
        .fold(input[0].0.as_ref(), |previous, (current, _)| {
            get_longer_str(previous, current)
        });

    input
        .iter()
        .map(|(key, value)| {
            format!(
                "{}{}: {}\n",
                " ".repeat(longest.len() - key.as_ref().len()),
                key.as_ref(),
                value.as_ref()
            )
        })
        .fold(String::new(), |a, b| a + &b)
}
