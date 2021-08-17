<p align="center">
  <img src="https://raw.githubusercontent.com/onnovalkering/brane/master/contrib/assets/logo.png" alt="logo" width="250"/>
  <h3 align="center">Programmable Orchestration of Applications and Networking</h3>
</p>

----

<span align="center">

  [![Audit status](https://github.com/onnovalkering/brane/workflows/Audit/badge.svg)](https://github.com/onnovalkering/brane/actions)
  [![CI status](https://github.com/onnovalkering/brane/workflows/CI/badge.svg)](https://github.com/onnovalkering/brane/actions)
  [![License: Apache-2.0](https://img.shields.io/github/license/onnovalkering/brane.svg)](https://github.com/onnovalkering/brane/blob/master/LICENSE)
  [![Coverage status](https://coveralls.io/repos/github/onnovalkering/brane/badge.svg)](https://coveralls.io/github/onnovalkering/brane)
  [![Release](https://img.shields.io/github/release/onnovalkering/brane.svg)](https://github.com/onnovalkering/brane/releases/latest)
  [![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.3890928.svg)](https://doi.org/10.5281/zenodo.3890928)

</span>

## Introduction

Regardless of the context and rationale, running distributed applications on geographically dispersed IT resources often comes with various technical and organizational challenges. If not addressed appropriately, these challenges may impede development, and in turn, scientific and business innovation. We have designed and developed Brane to support implementers in addressing these challenges. Brane makes use of containerization to encapsulate functionalities as portable building blocks. Through programmability, application orchestration can be expressed using intuitive domain-specific languages. As a result, end-users with limited or no programming experience are empowered to compose applications by themselves, without having to deal with the underlying technical details.

See the [documentation](https://onnovalkering.gitbook.io/brane) for more information.

## Contributing
If you're interrested in contributing, please read the [code of conduct](.github/CODE_OF_CONDUCT.md) and [contributing](.github/CONTRIBUTING.md) guide.

Bug reports and feature requests can be created in the [issue tracker](https://github.com/onnovalkering/brane/issues).

## Development
The latest version of [Rust](https://www.rust-lang.org), and the following system dependencies must be installed (assuming Ubuntu 20.04):

- build-essential
- cmake
- docker-compose
- docker.io
- libssl-dev
- musl-tools
- pkg-config

### Builds
To compile and test the complete project:
```
$ cargo build
$ cargo test
```

To build optimized versions of the binaries (`brane` and `branelet`):
```shell
$ make build-binaries
```

To build optimized versions of the services (Docker images):
```shell
$ make build-services
```
