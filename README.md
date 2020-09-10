<p align="center">
  <img src="tangram.svg" title="Tangram">
</p>

<h1 align="center">Tangram</h1>

[![GitHub Release](https://img.shields.io/github/release/tangram-hq/tangram.svg?style=flat-square)](https://github.com/tangram-hq/tangram/releases)
[![GitHub License](https://img.shields.io/badge/license-Tangram-blue)](https://github.com/pachyderm/pachyderm/blob/master/LICENSE)
[![CLA Assistant](https://cla-assistant.io/readme/badge/tangram-hq/tangram)](https://cla-assistant.io/tangram-hq/tangram)

Tangram is an all-in-one machine learning toolkit for developers. Watch the video at the link below to learn more.

[Watch the Video](tangramhq.com).

## Repo Structure

At the root, this repo is a Rust crate called `tangram` that is both a library and a binary. This means you can run `cargo run` from the root to run a debug build of the cli, or `cargo build --release` to produce a release build of both the cli and the library. The library has a C API, called libtangram, defined in [tangram.h](tangram.h) and implemented in [tangram.rs](tangram.rs), that is used by the libraries for each programming language to load models and make predictions.

The `www` directory contains the source code for the marketing and documentation website at tangramhq.com.

## Contributing

1. Install recent versions of [Rust](rust-lang.org), [Node](nodejs.org), and [Yarn](yarnpkg.org).
2. Clone this repo: `git clone https://github.com/tangram-hq/tangram`.
3. Run `yarn` to install npm dependencies.

Before submitting a pull request, please run `./scripts/check` at the root of the repository to confirm that your changes do not have any errors.
If your editor does not automatically observe the repo's `.rustfmt.toml` and `.prettierrc.json` files then please also run `./scripts/fmt`.

You can use whatever editor setup you prefer, but we use [Visual Studio Code](https://code.visualstudio.com/) with the [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer), [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint), and [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) extensions.

## License
