---
layout: default
title: Installation
nav_order: 3
description: "Installation"
permalink: /installation
---

# Installation
Brane has two installable components: the <abbr title="Command-line interface">CLI</abbr> and the backend.

## CLI
Both Linux and macOS are supported. When on Windows, use [Windows Subsystem for Linux (WSL)](https://docs.microsoft.com/en-us/windows/wsl/about).

#### Requirements:

- Docker, version 19.03 or higher, with the [BuildKit](https://github.com/docker/buildx#building) plugin (Tech Preview) enabled.
- On Linux, Docker should be [configured](https://docs.docker.com/engine/install/linux-postinstall/#manage-docker-as-a-non-root-user) to allow management as a non-root user, i.e. without `sudo`.

Download the pre-built binary for your platform from the [releases](https://github.com/onnovalkering/brane/releases) page and place it in a `$PATH` directory, with execute permission. It's recommended to use `brane` as the binary's name:   
```shell
$ curl -L github.com/onnovalkering/brane/releases/download/v0.1.0/brane-`uname` -o brane
$ chmod +x brane 
$ sudo mv brane /usr/local/bin/
```

__Note__: Alternatively, you can compile and install the <abbr title="Command-line interface">CLI</abbr> from [source code](https://github.com/onnovalkering/brane/tree/master/brane-cli) using [Cargo](https://doc.rust-lang.org/stable/cargo).

## Backend
The backend is installed, locally or remote, as either a Docker or Kubernetes deployment. 

For both deployments, you need a copy of the Brane repository:

```shell
$ git clone https://github.com/onnovalkering/brane.git
```

### Docker deployment
The Docker deployment makes use of [Docker Compose](https://docs.docker.com/compose). From the root of the Brane repository, run:
```shell
$ docker-compose -f deployment/docker/docker-compose.yml up -d
```

Then check if the deployment is successful using:
```shell
$ curl `hostname`:8080/health
```

### Kubernetes deployment
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