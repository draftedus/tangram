<p align="center">
  <img src="tangram.svg" title="Tangram">
</p>

[![GitHub Release](https://img.shields.io/github/release/tangram-hq/tangram.svg?style=flat-square)](https://github.com/tangram-hq/tangram/releases)

# Tangram

Tangram is an all-in-one machine learning toolkit for developers. Using existing tools, a lot of pieces have to come together to form a complete solution. Instead, Tangram provides all the tools, designed from the start to fit together perfectly, just like a Tangram. If you want to make predictions on behalf of your customers from your applications but have struggled to piece together the constellation of tools to make it happen, Tangram is for you. Watch the video at the link below to learn more.

[Watch the Video](tangramhq.com).

## Repo Structure

This repository is both a Cargo workspace and a Yarn workspace. Each folder in the root of the repository is either a Rust or NPM package. Below is a description of the most important packages:

### `cli`

### `www`

`www` is the marketing and documentation website at deployed to tangramhq.com.

At the root, this repo is a Rust crate called `tangram` that is both a library and a binary. This means you can run `cargo run` from the root to run a debug build of the cli, or `cargo build --release` to produce a release build of both the cli and the library. The library has a C API, called libtangram, defined in [tangram.h](tangram.h) and implemented in [tangram.rs](tangram.rs), that is used by the libraries for each programming language to load models and make predictions.

## Contributing

1. Install recent versions of [Rust](rust-lang.org), [Node](nodejs.org), and [Yarn](yarnpkg.org).
2. Clone this repo: `git clone https://github.com/tangram-hq/tangram`.
3. Run `yarn` to install npm dependencies.

Before submitting a pull request, please run `./scripts/check` at the root of the repository to confirm that your changes do not have any errors.
If your editor does not automatically observe the repo's `.rustfmt.toml` and `.prettierrc.json` files then please also run `./scripts/fmt`.

You can use whatever editor setup you prefer, but we use [Visual Studio Code](https://code.visualstudio.com/) with the [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer), [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint), and [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) extensions.

## License
