# RustShot: Cross-Platform Open Source Screenshot Utility

> This project is part of the Programmazione di sistema [02GRSOV] course of Politecnico di Torino

## Linux Dependencies

Debian / Ubuntu:

```sh
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev libgtk-3-dev libappindicator3-dev
```

## Creating executables

Install [`cargo-bundle`](https://github.com/burtonageo/cargo-bundle) and run:

```sh
cargo bundle
```

This command will create an executable bundle in `target/debug/bundle`. For
release mode just use the `--release` flag.

## License

MIT

## Third-party licenses

Icons provided by [Twitter](https://twemoji.twitter.com/).

Copyright 2020 Twitter, Inc and other contributors
Code licensed under the MIT License: <http://opensource.org/licenses/MIT>
Graphics licensed under CC-BY 4.0: <https://creativecommons.org/licenses/by/4.0/>
