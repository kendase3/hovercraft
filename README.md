# hovercraft

a vector-based 2d vehicle combat game

## Goals

- create a series of 1v1 battles with AI opponents with different loadouts
- model architecture on https://johanhelsing.studio/posts/extreme-bevy
- see also https://github.com/johanhelsing/extreme_bevy/tree/part-1

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for details.

Please read before proceeding to [Getting Started](#getting-started)

## Getting Started

### Prerequisites

- You'll need to install `rustup` which can be found [publicly here](https://rustup.rs/)

- Installing `rustup` also will automatically install `cargo` and add it to your `PATH`. Your shell may need to be restarted to see this change reflected

- See `dnf` file for required Fedora packages

### Build and Run

```
rustup target add wasm32-unknown-unknown
cargo install wasm-server-runner
cargo run
```

### Common Issues

You may get an issue stating there's a version mismatch of wasm-bindgen. Deleting the `Cargo.lock` file resolved the issue for me.

## License

Apache 2.0; see [`LICENSE`](LICENSE) for details.

## Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.

This is not an officially supported Google product.

## Build Dependencies

See https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md

### Docker dependencies

https://github.com/rust-windowing/winit/issues/3603#issuecomment-2204376719

## Work in Progress

Note that this game is currently just a lil' demo. But game coming soon!

### Janky Features

`cargo run` will run on a local port you can visit in your browser

`cargo run --target x86_64-unknown-linux-gnu` is not supported yet, but will run natively rather than on web

### Small demo game
To run a small demo game, run `cargo run --bin smol`.
