<p align="center">
  <img src="https://raw.githubusercontent.com/onnovalkering/brane/master/contrib/assets/logo.png" alt="logo" width="250"/>
</p>

# Brane
[![Build Status](https://github.com/onnovalkering/brane/workflows/CI/badge.svg)](https://github.com/onnovalkering/brane/actions)
[![License: Apache-2.0](https://img.shields.io/github/license/onnovalkering/brane.svg)](https://github.com/onnovalkering/brane/blob/master/LICENSE)
[![Coverage Status](https://coveralls.io/repos/github/onnovalkering/brane/badge.svg)](https://coveralls.io/github/onnovalkering/brane)
[![Release](https://img.shields.io/github/release/onnovalkering/brane.svg)](https://github.com/onnovalkering/brane/releases/latest)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.3890928.svg)](https://doi.org/10.5281/zenodo.3890928)

Brane provides a programmatic approach to constructing research infrastructures that is intuitive and easy to use, yet is expressive enough to capture and control the entire, distributed, technical stack. The programming model is based on the separation of concerns principle. For each level of the technical stack, and associated roles, different tooling and abstractions are provided. As a result, top-level applications can be written in a domain-specific language by domain scientists, while underlying routines are implemented and optimised by the relevant experts.

See the [documentation](https://onnovalkering.github.io/brane) for more information.

## Contributing
If you're interrested in contributing, please read the [code of conduct](.github/CODE_OF_CONDUCT.md) and [contributing](.github/CONTRIBUTING.md) guide.

Bug reports and feature requests can be created in the [issue tracker](https://github.com/onnovalkering/brane/issues).

## Development
The following system dependencies must be installed (assuming Ubuntu 20.04):

- build-essential
- cmake
- libpq-dev
- libssl-dev
- pkg-config

### Compiling
To compile all components:
```shell
cargo build
```

To create optimized release version of the binaries ([brane-cli](brane-cli) and [brane-init](brane-init)):
```shell
make binaries
```

To create Docker images of the services ([brane-api](brane-api), [brane-ide](brane-ide), and [brane-loop](brane-loop)):
```shell
make docker
```

### Running
To start and stop a local development instance:
```shell
make start
make stop
```
These commands rely on `tmux` to simultaneously start and stop multiple services.
