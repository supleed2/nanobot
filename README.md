# nanobot

Discord bot written with [serenity-rs/poise](https://github.com/serenity-rs/poise) and [tokio-rs/axum](https://github.com/tokio-rs/axum), designed to run on [Shuttle](https://www.shuttle.rs). It allows users to be de-anonymised and automatically verified for entry to a Discord server.

## License

This repo is under the ISC license, with the following exception.

- SQLite Fuzzy extension from the [sqlean](https://github.com/nalgeon/sqlean) project, included here for ease of building the docker image. It is not covered by the license on this repository and the copyright for this extension is available [in the repo](https://github.com/nalgeon/sqlean).
  - Excerpt from the repo: Copyright 2021-2025 [Anton Zhiyanov](https://antonz.org/), [Contributors](https://github.com/nalgeon/sqlean/graphs/contributors) and [Third-party Authors](https://github.com/nalgeon/sqlean/blob/main/docs/third-party.md).
