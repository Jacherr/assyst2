# assyst-proc-macro

This crate defines proc macros, used primarily by the `assyst-core` crate for its `#[command]` macro.

### `#[command]`
This macro allows you to write bot commands (independent of the source: gateway or interaction) as regular async functions.

The first parameter must always be the `CommandCtxt`, followed by any number of argument types.
`Rest` must be the last argument, if present. For example, the `-remind 1h do something` can be defined as:
```rs
#[command(
    name = "remind",
    aliases = ["reminder"],
    description = "get reminders or set a reminder, time format is xdyhzm (check examples)",
    access = Availability::Public,
    cooldown = Duration::from_secs(2)
)]
pub fn remind(ctxt: CommandCtxt<'_>, when: Time, text: Rest) -> anyhow::Result<()> {
    // ...
    Ok(())
}
```
The macro takes some metadata about the command (description, availablity, etc.) and generates a struct that implements the `Command` trait, in which it calls `::parse()` on each of the provided types (`Time`, `Rest`) and finally passes it to the annotated function.

For even more details (e.g. its exact expansion), take a look at the code. There's documentation on the proc macro function, too.
