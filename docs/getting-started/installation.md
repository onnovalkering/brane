---
description: Install dependencies and setup a Brane instance.
---

# Installation

Brane has three installable components: a command-line interface (CLI); a Brane instance (backend); and a custom [JupyterLab](https://jupyterlab.readthedocs.io/en/stable) IDE. A complete installation is recommended, but might not be needed by all users.&#x20;

{% hint style="info" %}
Both Linux and macOS are supported. On Windows, try to use [Windows Subsystem for Linux](https://docs.microsoft.com/en-us/windows/wsl/about).
{% endhint %}

**Requirements:**

* Docker, version 19.03 or higher, with the (experimental) [BuildKit](https://github.com/docker/buildx#building) plugin installed and enabled.
* On Linux, Docker should be [configured](https://docs.docker.com/engine/install/linux-postinstall/#manage-docker-as-a-non-root-user) to allow management as a non-root user, i.e., without `sudo`.
* Using the automated build and setup scripts requires `curl`, `git`, [`jq`](https://stedolan.github.io/jq), and `make` (optional).

## Command-line interface

Download the pre-built binary appropriate for your platform from the [releases](https://github.com/onnovalkering/brane/releases) page and place it, with execute permissions,  in a `$PATH` directory (e.g., `$HOME/.local/bin`). See the script below.

{% hint style="info" %}
For consistency, it's recommended to rename the downloaded binary to `brane`.
{% endhint %}

{% code title="./contrib/scripts/install-brane-cli.sh" %}
```bash
#!/usr/bin/env bash
set -euo pipefail

# Determine the latest version (requires `jq`).
VERSION=$(\
    curl -L -s "api.github.com/repos/onnovalkering/brane/tags" \
  | jq -r '.[0].name' \
)

# Download the appropriate binary and save it as `brane`.
curl "github.com/onnovalkering/brane/releases/download/$VERSION/brane-`uname`" \
     -L -s -o brane
     
TARGET_DIR="$HOME/.local/bin"
mkdir -p $TARGET_DIR

# Add execute permissions and place it in the target directory.
chmod +x brane
mv brane $TARGET_DIR

# Check if target directory is in $PATH.
if [[ ! :$PATH: == *:"$TARGET_DIR":* ]] ; then
     echo "WARN: Please add '$TARGET_DIR' to \$PATH."
fi
```
{% endcode %}

It's also possible to install the CLI directly from the source code (requires [`cargo`](https://doc.rust-lang.org/cargo) and [`rustfmt`](https://github.com/rust-lang/rustfmt)):

```
cargo install --git http://github.com/onnovalkering/brane brane-cli
```

If you encouter problems running the `brane` binary, you can try a build based on a lower [`glibc`](https://www.gnu.org/software/libc) version. This build can be obtained by copying it out of a Docker image (automatically build from this [Dockerfile](https://github.com/onnovalkering/brane/blob/develop/Dockerfile.cli)):

```bash
docker run --rm \
    --entrypoint "/bin/sh" \
    -v $(pwd):/binary \
    onnovalkering/brane-cli \
    -c "cp /brane /binary"
```

## Brane instance

First, clone the Brane repository to acquire the required source code files:

```bash
$ git clone https://github.com/onnovalkering/brane
```

A Brane instance relies on an `infra.yml` file that describes the various compute resources that the instance can use. As a starting point, copy the `infra-local-vm.yml` file, as `infra.yml`, from the `./contrib/config` directory to the repository's root. With this configuration file, the machine where Brane is running will also be used as the compute target. See the [testbed](broken-reference) page for a more elaborate configuration.&#x20;

```
$ cp -iv ./contrib/config/infra-local-vm.yml ./infra.yml
```

A `secrets.yml` file is also required to be present in the root of the repository. Currently, we don't have any secrets to store. Therefore, we can just create a dummy `secrets.yml` file:

```
$ echo "dummy: secret" >> secrets.yml
```

Then, an automated script can be used to start the Brane instance:

```bash
$ make start-instance
```

The `health` endpoint of the Brane API can be used to check if the Brane instance is running correclty:

```
$ curl http://localhost:8080/health
OK!
```

{% hint style="warning" %}
Brane is not yet production-ready, please don't run Brane with publically exposed ports.
{% endhint %}
