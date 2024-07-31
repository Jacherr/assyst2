<div align="center">
    <img src="https://cdn.discordapp.com/avatars/571661221854707713/a23ec18f81e3d182291471f64685da5f.png?size=128"/><br>
</div>

# <div align="center"> Assyst </div>

<div align="center">

![Discord](https://img.shields.io/discord/1099115731301449758?color=7289DA)
![GitHub](https://img.shields.io/github/license/jacherr/assyst2)
[![Discord Bots](https://top.gg/api/widget/servers/571661221854707713.svg?noavatar=true)](https://top.gg/bot/571661221854707713)
[![Discord Bots](https://top.gg/api/widget/status/571661221854707713.svg?noavatar=true)](https://top.gg/bot/571661221854707713)

</div>

Assyst is a multi-purpose Discord bot with a focus on image processing and manipulation, custom commands via a tag parser, and other unique features. A more detailed overview of the Assyst feature-set can be located on the [Top.gg listing page for Assyst](https://top.gg/bot/571661221854707713).

Assyst is split into a number of separate crates, as described below.

## Binaries
- assyst-core: Main command-handling process. Also contains logic for the parsing of message-based commands.
- assyst-gateway: Connects to the Discord WebSocket gateway to receive messages, which are then forwarded to assyst-core for processing.
- assyst-cache: Independent cache process designed to hold some caching information.

## Libraries
- assyst-common: Utilities, structures, and functions shared throughout the entire Assyst ecosystem.
- assyst-tag: Tag parser and handler.
- assyst-database: Interfaces with PostgreSQL, for database purposes.
- assyst-webserver: Web server designed to handle webhooking, such as vote processing for Discord bot list websites, as well as Prometheus metrics.
- assyst-proc-macro: General purpose [procedural macros] <sub>(currently just a macro for command setup)</sub>

### Note: assyst-core will likely be split into more, smaller, crates in the future.

[Procedural macros]: https://doc.rust-lang.org/reference/procedural-macros.html

For more information on each crate, refer to the README.md file for the crate.

Each binary is ran as an independent process on the same host machine. Each binary communicates through the use of Unix-like pipes. For more information, please refer to the README.md file for the relevant crate.

## Contributing

All contributions - both issues and pull requests - are greatly appreciated. Contributions are done on a fairly loose basis. The easiest way to begin contributing is to first understand the structure of Assyst - this can be done initially by understanding all individual crates by reading their READMEs. If you have any questions, feel free to open an issue. All issues are free to be tackled by anyone.

## Self-hosting

Self-hosting is not yet supported for this version of Assyst, since it is not yet considered production-ready. Self-hosting may be supported with release 1.0.0. \
However, for completeness, the entire tech stack of Assyst is as follows:
 - Rust, as well as Cargo for building.
 - PostgreSQL. Database format TBA.
 - Flux, which has [its own set of requirements](https://github.com/jacherr/flux?tab=readme-ov-file#prerequisites)
 - [youtube-dlp](https://github.com/yt-dlp/yt-dlp)
 - fake-eval service (currently closed source).
 - CDN (filer) service (currently closed course).
 - Optionally, Grafana and Prometheus for graphs and logging. A template for this may be made available eventually. If you would like it, open an issue.

## Acknowledgements

Special thanks to [y21](https://github.com/y21) and [Mina](https://github.com/trueharuu) for their invaluable help and contributions towards this version of Assyst. \
Thank you to the team developing [cobalt.tools](https://github.com/imputnet/cobalt) for creating such a versatile and easy-to-use downloading tool. \
Thank you to the countless developers of the libraries and programs powering both Assyst and Flux, in particular:
- [Tokio](https://github.com/tokio/tokio) - the asynchronous runtime that Assyst uses,
- [Twilight](https://github.com/twilight-rs/twilight) - the Discord API library that Assyst communicates to Discord with,
- [Image](https://github.com/image-rs/image) - the primary library providing image decoding, encoding, and editing functionality to Flux,
- [FFmpeg](https://ffmpeg.org) - simply the best multimedia processing tool ever made,
- [gegl](https://gegl.org) - providing a bunch of handy image manipulation tools.

