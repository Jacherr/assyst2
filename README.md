<div align="center">
    <img src="https://cdn.discordapp.com/avatars/571661221854707713/a23ec18f81e3d182291471f64685da5f.png?size=128"/><br>
</div>

# <div align="center"> Assyst2 </div>

<div align="center">

![Discord](https://img.shields.io/discord/1099115731301449758?color=7289DA)
![GitHub](https://img.shields.io/github/license/jacherr/assyst2)

</div>

Complete rewrite of Assyst, open-sourced. Designed to integrate both with traditional text-based commands as well as slash commands.

Assyst is a multi-purpose Discord bot with a focus on image processing and manipulation, custom commands via a tag parser, and other unique features. A more detailed overview of the Assyst feature-set can be located on the [Top.gg listing page for Assyst](https://top.gg/bot/571661221854707713).

Assyst2 is split into a number of separate crates, as described below.

## Binaries
- assyst-core: Main command-handling process. Also contains logic for the parsing of message-based commands.
- assyst-gateway: Connects to the Discord WebSocket gateway to receive messages, which are then forwarded to assyst-core for processing.
- assyst-slash-client: HTTP client designed to handle slash commands.
- assyst-cache: Independent cache process designed to hold some caching information.

## Libraries
- assyst-common: Utilities, structures, and functions shared throughout the entire Assyst ecosystem.
- assyst-tag: Tag parser and handler.
- assyst-database: Interfaces with PostgreSQL, for database purposes.
- assyst-webserver: Web server designed to handle webhooking, such as vote processing for Discord bot list websites, as well as Prometheus metrics.
- assyst-proc-macro: General purpose [procedural macros] <sub>(currently just a macro for command setup)</sub>

[Procedural macros]: https://doc.rust-lang.org/reference/procedural-macros.html

For more information on each crate, refer to the README.md file for the crate.

Each binary is ran as an independent process on the same host machine. Each binary communicates through the use of Unix-like pipes, a.k.a., the [UnixStream](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html) structure in [Tokio](https://crates.io/crates/tokio). For more information, please refer to the README.md file for the relevant crate.

While this repository can technically be self-hosted, the core image processing components are not open-source and as a result will not function. No support will be given for the self-hosting of this bot. However, if you would like to contribute, or need support with operating the bot, please send a message in the [Discord support server](https://discord.gg/brmtnpxbtg). All contributions, especially in the early stages in development, are greatly appreciated.

Special thanks to [y21](https://github.com/y21) and [Mina](https://github.com/trueharuu) for their help in the development of this rewrite.
