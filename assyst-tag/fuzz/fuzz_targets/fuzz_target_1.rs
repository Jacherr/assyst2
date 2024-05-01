#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: (&str, Vec<String>)| {
    let (data, args) = data;
    let args = args.iter().map(|v| v.as_str()).collect::<Vec<_>>();
    if let Err(err) = assyst_tag::parse(
        data,
        &args,
        assyst_tag::parser::ParseMode::StopOnError,
        assyst_tag::NopContext,
    ) {
        assyst_tag::errors::format_error(data, err);
    }
});
