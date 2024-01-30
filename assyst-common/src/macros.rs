#[macro_export]
macro_rules! ok_or_break {
    ($expression:expr) => {
        match $expression {
            Ok(v) => v,
            Err(_) => break,
        }
    };
}

#[macro_export]
macro_rules! ok_or_continue {
    ($expression:expr) => {
        match $expression {
            Ok(v) => v,
            Err(_) => continue,
        }
    };
}

#[macro_export]
macro_rules! tracing_init {
    () => {{
        use assyst_common::ansi::Ansi;
        use time::format_description;
        use tracing_subscriber::fmt::time::UtcTime;
        use tracing_subscriber::EnvFilter;

        let filter = EnvFilter::from_default_env().add_directive("twilight_gateway=info".parse().unwrap());
        let description = "[year]-[month]-[day] [hour]:[minute]:[second]";

        tracing_subscriber::fmt()
            .with_timer(UtcTime::new(format_description::parse(description).unwrap()))
            .with_line_number(true)
            .with_env_filter(filter)
            .init();
    }};
}
