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

/// Generates a table given a list of tuples containing strings
pub fn generate_table<T: AsRef<str>>(input: &[(T, T)]) -> String {
    let longest: &str = input.iter().fold(input[0].0.as_ref(), |previous, (current, _)| {
        get_longer_str(previous.as_ref(), current.as_ref())
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

/// Generates a list given a list of tuples containing strings
pub fn generate_list<K: AsRef<str>, V: AsRef<str>>(key_name: &str, value_name: &str, values: &[(K, V)]) -> String {
    let longest = get_longer_str(
        key_name,
        values.iter().fold(values[0].0.as_ref(), |previous, (current, _)| {
            get_longer_str(previous, current.as_ref())
        }),
    );

    let mut output = format!(
        " {4}{}\t{}\n {4}{}\t{}",
        key_name,
        value_name,
        "-".repeat(key_name.len()),
        "-".repeat(value_name.len()),
        " ".repeat(longest.len() - key_name.len()),
    );

    let formatted_values = values
        .iter()
        .map(|(k, v)| {
            format!(
                " {}{}\t{}",
                " ".repeat(longest.len() - k.as_ref().chars().count()),
                k.as_ref(),
                v.as_ref()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    output = format!("{}\n{}", output, formatted_values);

    output
}
