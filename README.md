# Brane
[![Build Status](https://github.com/onnovalkering/brane/workflows/CI/badge.svg)](https://github.com/onnovalkering/brane/actions)
[![License: Apache-2.0](https://img.shields.io/github/license/onnovalkering/brane.svg)](https://github.com/onnovalkering/brane/blob/master/LICENSE)
[![Coverage Status](https://coveralls.io/repos/github/onnovalkering/brane/badge.svg)](https://coveralls.io/github/onnovalkering/brane)
[![Release](https://img.shields.io/github/release/onnovalkering/brane.svg)](https://github.com/onnovalkering/brane/releases/latest)
[![DOI](https://zenodo.org/badge/258514017.svg)](https://zenodo.org/badge/latestdoi/258514017)

Brane provides a programmatic approach to constructing workflows and research infrastructures that is intuitive and easy to use, yet is expressive enough to capture and control the entire, distributed, technical stack. For each level of the technical stack different tooling and abstractions are provided. As a result, workflows can be written in a high-level language directly by domain scientists, while invidual workflow steps can be implemented separately, in an isolated manner, by the relevant expert.

See the [documentation](https://onnovalkering.github.io/brane) for more information.

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

To create release version of the binaries (`cli` and `init`):
```shell
make binaries
```

To create the Docker images: (`brane-api`, `brane-ide`, and `brane-loop`):
```shell
make docker
```

### Running
To start a development instance from the source code:

```shell
make start
```

This will create a new `tmux` session with a pane for different components.

<img src="https://raw.githubusercontent.com/onnovalkering/brane/develop/docs/assets/img/tmux-dev.png" alt="preview" width="400"/>
