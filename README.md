<p align="center">
	<img src="tangram.svg" title="Tangram">
</p>

# Tangram

Tangram is an all-in-one machine learning toolkit for developers. Here's the TLDR:

1. Install the `tangram` cli: [Install Instructions](https://www.tangramhq.com/docs/install).
2. Train a machine learning model from a CSV file: `tangram train --file heart-disease.csv --target diagnosis`. The cli automatically performs feature engineering, trains a number of models with a range of hyperparameter settings, and writes the best one to `heart-disease.tangram` in the current directory. Your data stays secure because all training happens on your laptop or your own server. If you want more control, you can configure training with a YAML config file.
3. Use one of the language libraries to load your model and make predictions from your code. Prediction happens in process, so predictions are fast and data doesn't go over the network. Go, JavaScript, Python, and Ruby are available now. C/C++, C#, Java, PHP, and Rust are coming soon.
4. Run `tangram app --model heart-disease.tangram`. This starts a web app where you can:

- View stats and metrics showing how your model performed on the test set.
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

1. Install recent versions of [Rust](rust-lang.org), [Node](nodejs.org), and [Yarn](yarnpkg.org) on Linux, macOS, or Windows.
2. Clone this repo and `cd` into it.
3. Run `yarn` to install npm dependencies.
4. Run `cargo run` to run a debug build of the cli. If you are working on the app, install [watchexec](github.com/watchexec/watchexec) and run `./scripts/dev`, which runs the cli with the `app` subcommand under a file watcher.

Before submitting a pull request, please run `./scripts/fmt` and `./scripts/check` at the root of the repository to confirm that your changes are formatted correctly and do not have any errors.

### Tips

- To get faster incremental compile times, install [lld](https://www.archlinux.org/packages/extra/x86_64/lld/) and add `build.rustflags = ["-C", "link-arg=-fuse-ld=lld"]` to `~/.cargo/config`.

- To save your SSD from a premature death, set up a RAM disk for Cargo's `target` folder on linux by adding `none /path/to/tangram/target tmpfs rw,relatime 0 0` to `/etc/fstab` and rebooting.

- For good IDE support, use [Visual Studio Code](https://code.visualstudio.com/) with the [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer), [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint), and [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) extensions.

## Repository Structure

This repository is both a Cargo workspace and a Yarn workspace. Almost every folder in the root of the repository is either a Rust crate or an NPM package. Below is a description of the most important folders:

- [`core`](core): This folder contains the `tangram_core` crate that defines the model file format and automated machine learning functionality. It is used by the `tangram_cli` crate to train a model, and by the `libtangram` crate to expose its functionality as a C api to the language libraries.

- [`linear`](linear): This folder contains the `tangram_linear` crate that implements linear machine learning models.

- [`tree`](tree): This folder contains the `tangram_tree` crate that implements tree machine learning models.

- [`app`](app): This folder contains the `tangram_app` crate that implementats the reporting and monitoring web app. See `run()` in [app/lib.rs](app/lib.rs) for the entrypoint.

- [`cli`](cli): This folder contains the `tangram_cli` crate. It uses the `tangram_core` crate to train a model, and the `tangram_app` crate to run the reporting and monitoring web app. See `main()` in [cli/main.rs](cli/main.rs) for the entrypoint.

- [`libtangram`](libtangram): This folder contains the `libtangram` crate which produces the static and dynamic C libraries that are used by the language libraries to make predictions.

- [`languages`](languages): This folder contains the libraries for making predictions in each language. Each library has a README with more information.

- [`www`](www): This folder contains the source for the marketing and documentation website deployed to https://www.tangramhq.com.

## License

Most of this repository is MIT licensed, except for the `core` and `app` folders, which are presently unlicensed. We plan to make the `core` folder available under a community license that is similar to the MIT license, except that it will restrict you from using it to make a competing product. We plan to make the `app` folder free to use for an individual, but require a paid license to use as part of a team. Our pricing is simple and public: https://www.tangramhq.com/pricing.
