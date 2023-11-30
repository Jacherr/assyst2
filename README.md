# Assyst2

Complete rewrite of Assyst, open-sourced. Designed to integrate both with traditional text-based commands as well as slash commands.

Assyst is a multi-purpose bot with a focus on image processing and manipulation, custom commands via a tag parser, and other unique features. A more detailed overview of the Assyst feature-set can be located on the [Top.gg listing page for Assyst](https://top.gg/bot/571661221854707713).

Assyst2 is split into a number of separate crates, as described below.

## Binaries
- assyst-core: Main command-handling process. Abstracted from the details of gateway and slash command implementations.
- assyst-gateway: Connects to the Discord WebSocket gateway to receive and send events. 
- assyst-slash-client: HTTP client designed to handle slash commands.
- assyst-cache: Independent cache process designed to hold some caching information.

## Libraries
- assyst-common: Utilities, structures, and function shared throughout the entire Assyst ecosystem.
- assyst-tag: Tag parser and handler.
- assyst-database: Interfaces with PostgreSQL, for database purposes.
- assyst-webserver: Web server designed to handle webhooking, such as vote processing for Discord bot list websites.
- assyst-logger: Logger crate with functions designed to provide a standardised logging format.

For more information on each crate, refer to the README.md file for the crate.

Each binary is ran as an independent process on the same host machine. Each binary communicates through the use of Unix-like pipes, a.k.a., the [UnixStream](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html) structure in [Tokio](https://crates.io/crates/tokio). For more information, please refer to the README.md file for the relevant crate.

While this repository can technically be self-hosted, the core image processing components are not open-source and as a result will not function. No support will be given for the self-hosting of this bot. However, if you would like to contribute, or need support with operating the bot, please send a message in the [Discord support server](https://discord.gg/brmtnpxbtg). All contributions, especially in the early stages in development, are greatly appreciated.

Special thanks to [y21](https://github.com/y21) and [Mina](https://github.com/trueharuu) for their help in the development of this rewrite.