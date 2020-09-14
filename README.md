<p align="center">
  <img src="tangram.svg" title="Tangram">
</p>

[![GitHub Release](https://img.shields.io/github/release/tangram-hq/tangram.svg?style=flat-square)](https://github.com/tangram-hq/tangram/releases)

# Tangram

Tangram is an all-in-one machine learning toolkit for developers. Here's the TLDR:

1. Install the `tangram` cli: [Install Instructions](https://www.tangramhq.com/docs/install).
2. Train a machine learning model from a CSV file: `tangram train --file heart-disease.csv --target diagnosis`. The cli automatically performs feature engineering, trains a number of models with a range of hyperparameter settings, and writes the best one to `heart-disease.tangram` in the current directory. If you want more control, you can configure training with a YAML config file.
3. Use one of the language libraries to load your model and make predictions from your code. Prediction happens in process, so predictions are fast and customer data doesn't go over the network. Go, JavaScript, Python, and Ruby are available now. C/C++, C#, Java, PHP, and Rust are coming soon.
4. Run `tangram app --model heart-disease.tangram`. This starts a web app where you can:

- View stats on your training dataset.
- View metrics showing how your model performed on the test set.
- Tune your model to get the best performance.
- Make example predictions and view detailed explanations.
- Set up production monitoring so you can view stats, metrics, and explanations of real-world predictions.

Watch the video below to learn more.

<p align="center">
  <img src="tangram.svg" title="Tangram">
</p>

## Getting Started

[Follow the tutorial](https://www.tangramhq.com/docs).

## Contributing

1. Install recent versions of [Rust](rust-lang.org), [Node](nodejs.org), and [Yarn](yarnpkg.org) on linux, macOS, or Windows on x86_64.
2. Clone this repo and `cd` into it.
3. Run `yarn` to install npm dependencies.
4. Run `cargo run` to run a debug build of the cli. If you are working on the app, run `./scripts/dev`, which runs the cli with the `app` subcommand under a file watcher.

Before submitting a pull request, please run `./scripts/fmt` and `./scripts/check` at the root of the repository to confirm that your changes are formatted correctly and do not have any errors. You can use whatever editor setup you prefer, but we use [Visual Studio Code](https://code.visualstudio.com/) with the [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer), [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint), and [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) extensions.

## Repository Structure

This repository is both a Cargo workspace and a Yarn workspace. Almost every folder in the root of the repository is either a Rust crate or an NPM package. Below is a description of the most important packages:

### `core`

This folder contains the `tangram_core` crate that defines the model file format, machine learning algorithms, automated machine learning functionality. It is used by the `tangram_cli` crate to train a model, and by the `libtangram` crate to expose its functionality as a C api for the language libraries.

### `app`

This folder contains the `tangram_app` crate that contains the implementation of the web app.

### `cli`

This crate produces the `tangram` cli. It uses the `app` . See `main()` in `cli/main.rs` for the entrypoint.

### `libtangram`

This crate produces the `libtangram` static and dynamic C libraries that are used by the language libraries to make predictions.

### `languages`

This folder contains the libraries for making predictions in each language. Each library has a `README.md` with more information.

### `www`

This package holds the source for the marketing and documentation website deployed to https://www.tangramhq.com.

## License

We haven't yet spoken to a lawyer, so this repository presently has no license. However, the plan is to make the entire repo except the `app` folder free to use in any way except to create a competing product. The app will be free to use for an individual, but require a paid license to be used by a team. We are committed to simple public pricing. See here: https://www.tangramhq.com/pricing.
