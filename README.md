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

### Build and Run

```
rustup target add wasm32-unknown-unknown
cargo install wasm-server-runner
cargo build
cargo run
```

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

I'm still iterating on even basic, outer things for this game. Currently, one must manually run wasm_server_runner on the target wasm, rather than running it via cargo run.
