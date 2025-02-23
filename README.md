# KOSync Light

A lightweight implementation of the [Koreader Sync Server](https://github.com/koreader/koreader-sync-server/) in Rust. KOSync Light stores data as files in a configurable directory and exposes the sync server API. It is meant to be run on the ebook reader itself paired with [Syncthing](https://github.com/syncthing/syncthing) or similar software which would synchronize the data directory across devices.

## Installation

TODO

## Screenshots

TODO

## Building

### Dependencies

You will need the rust language toolchain ([instructions](https://www.rust-lang.org/tools/install)).

To build the project run:

```sh
cargo build --release
```

If you are cross-compiling e.g. for a Kindle Paperwhite Gen 4 (ARMv7) it's recommended to use LLVM's `lld` and link against musl. Add the target in rustup:

```sh
rustup target add armv7-unknown-linux-musleabihf
```

and then run:

```sh
cargo build --release --target=armv7-unknown-linux-musleabihf
```

The project already includes configuration under `.cargo/config.toml` to default to `lld` for `armv7-unknown-linux-musleabihf`.
