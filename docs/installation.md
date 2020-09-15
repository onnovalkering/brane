---
layout: default
title: Installation
nav_order: 3
description: "Installation"
permalink: /installation
---

# Installation
Both Linux and macOS are supported. When on Windows, use [Windows Subsystem for Linux (WSL)](https://docs.microsoft.com/en-us/windows/wsl/about).

A complete Brane installation consists of a CLI application and an instance deployment.

## CLI
The only prerequisite for the CLI is Docker (19.03+) with the [BuildKit](https://github.com/docker/buildx) plugin:
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

To install the CLI, download the pre-built binary for your platform from the [releases](https://github.com/onnovalkering/brane/releases) page and place it in a `$PATH` directory, with execute permission. It's convenient to use `brane` as the name of the binary:   
```shell
$ curl -L github.com/onnovalkering/brane/releases/download/v0.1.0/brane-`uname` -o brane
$ chmod +x brane && mv brane /usr/local/bin/
```

Alternatively, you can compile and install the CLI from the [source code](https://github.com/onnovalkering/brane/tree/master/brane-cli) using [Cargo](https://doc.rust-lang.org/stable/cargo).

## Instance
Brane instances are composites of several services, deployable using Docker or Kubernetes.

For both deployments, you need a copy of the Brane repository:

```shell
$ git clone https://github.com/onnovalkering/brane.git
```

The services that make up a Brane instance:

| Service   | Port      | Public |
|:----------|:----------|:-------|
| [Apache Kafka](http://kafka.apache.org)     | 9092      | No     |
| [Brane API](https://github.com/onnovalkering/brane)       | 8080      | Yes    |
| [Brane VM](https://github.com/onnovalkering/brane)       | -      | No    |
| [Docker Registry](https://docs.docker.com/registry)  | 5000      | Yes    |
| [HashiCorp Vault](https://www.vaultproject.io)     | 8200      | Yes    |
| [JupyterLab](https://jupyter.org)   | 8888      | Yes    |
| [NLeSC Xenon](http://xenon-middleware.github.io/xenon/)     | 50051     | No     |
| [PostgreSQL](https://www.postgresql.org)  | 5432      | No     |
| [Redis](https://redis.io)     | 6379      | No     |

### Docker
The Docker deployment makes use of [Docker Compose](https://docs.docker.com/compose). Depending on your plaform, it might not be included in your Docker installation. Please see Docker's [documentation](https://docs.docker.com/compose/install/#install-compose) for installation instructions.

From the root of the Brane repository, run:
```shell
$ docker-compose -f deployment/docker/docker-compose.yml up -d
```

Then check if the deployment is successfull using:
```shell
$ curl `hostname`:8080/health
```

### Kubernetes
Brane's Kubernetes deployment relies on [Helm](https://helm.sh) – _"The package manager for Kubernetes"_. Please see Helm's [documentation](https://helm.sh/docs/intro/install/) for the installation instructions appropriate for your platform.

First, create and switch to a new namespace for the Brane instance:

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