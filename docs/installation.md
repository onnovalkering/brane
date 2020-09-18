---
layout: default
title: Installation
nav_order: 3
description: "Installation"
permalink: /installation
---

# Installation


## CLI
Both Linux and macOS are supported. When on Windows, use [Windows Subsystem for Linux (WSL)](https://docs.microsoft.com/en-us/windows/wsl/about).

The only prerequisite for the <abbr title="Command-Line Interface">CLI</abbr> is Docker (19.03+) with the [BuildKit](https://github.com/docker/buildx) plugin:
```shell
$ export DOCKER_BUILDKIT=1
$ docker build --platform=local -o . git://github.com/docker/buildx
$ mkdir -p ~/.docker/cli-plugins
$ mv buildx ~/.docker/cli-plugins/docker-buildx
$ docker buildx create --use
```

On Linux, Docker should be [configured](https://docs.docker.com/engine/install/linux-postinstall/#manage-docker-as-a-non-root-user) to allow use by a non-root user (i.e. without `sudo`):
```shell
$ sudo groupadd docker
$ sudo usermod -aG docker $USER
```

To install the <abbr title="Command-Line Interface">CLI</abbr>, download the pre-built binary for your platform from the [releases](https://github.com/onnovalkering/brane/releases) page and place it in a `$PATH` directory, with execute permission. It's convenient to use `brane` as the name of the binary:   
```shell
$ curl -L github.com/onnovalkering/brane/releases/download/v0.1.0/brane-`uname` -o brane
$ chmod +x brane && mv brane /usr/local/bin/
```

Alternatively, you can compile and install the <abbr title="Command-Line Interface">CLI</abbr> from the [source code](https://github.com/onnovalkering/brane/tree/master/brane-cli) using [Cargo](https://doc.rust-lang.org/stable/cargo).

## Backend
The backend can be deployed using Docker or Kubernetes. 

For both deployments, you need a copy of the Brane repository:

```shell
$ git clone https://github.com/onnovalkering/brane.git
```

### Docker
The Docker deployment makes use of [Docker Compose](https://docs.docker.com/compose). From the root of the Brane repository, run:
```shell
$ docker-compose -f deployment/docker/docker-compose.yml up -d
```

Then check if the deployment is successfull using:
```shell
$ curl `hostname`:8080/health
```

### Kubernetes
The Kubernetes deployment is based on [Helm](https://helm.sh). First, create a new namespace for Brane:

```shell
$ kubectl create namespace "brane"
$ kubectl config set-context $(kubectl config current-context) --namespace "brane"
```

To start the deployment, run the following from the root of the repository:
```shell
$ export HOSTNAME="<insert public K8s hostname or IP address>"
$ helm install brane deployment/kubernetes --set global.hostname=$HOSTNAME
```

Check if the deployment is successfull:
```shell
$ curl $HOSTNAME:8080/health
```