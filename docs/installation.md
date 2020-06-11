---
layout: default
title: Installation
nav_order: 2
description: "Installation"
permalink: /installation
---

# Installation
Only the Linux platform (x86_64) is currently supported. Support for macOS is coming soon.

A complete Brane installation consists of a CLI application and an instance deployment. 

## CLI
The only prerequisite for the CLI is Docker (19.03+) with the [BuildKit](https://github.com/docker/buildx) plugin:
```shell
$ export DOCKER_BUILDKIT=1
$ docker build --platform=local -o . git://github.com/docker/buildx
$ mkdir -p ~/.docker/cli-plugins
$ mv buildx ~/.docker/cli-plugins/docker-buildx
```

Futhermore, Docker should be [configured](https://docs.docker.com/engine/install/linux-postinstall/#manage-docker-as-a-non-root-user) to allow use by a non-root user (i.e. without `sudo`):
```shell
$ sudo groupadd docker
$ sudo usermod -aG docker $USER
```

To install the CLI itself, simply download the prebuild binary from the [releases](https://github.com/onnovalkering/brane/releases) page and place it in a `$PATH` directory, with execute permission. It's recommended to use `brane` as the name of the binary.
```shell
$ curl -LO https://github.com/onnovalkering/brane/releases/download/v0.1.0/brane-cli
$ chmod +x brane-cli 
$ mv brane-cli /usr/local/bin/brane
```

Alternatively, you can install the CLI from the [source code](https://github.com/onnovalkering/brane) using [Cargo](https://doc.rust-lang.org/stable/cargo/).

## Instance
Instances can be deployed locally with Docker or remotely with Kubernetes (recommended).

### Local
...

### Remote
...