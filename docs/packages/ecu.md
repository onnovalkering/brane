---
layout: default
title: ECU
parent: Packages
nav_order: 3
---

# Explicit Container Usage
A package can be built based on arbitrary code. The usage of the code must be specified explicitly in a configuration file, conventionally called `container.yml`, using Brane's ECU specification.

```shell
$ brane build container.yml
```

Point the `build` command of the Brane CLI to the file containing the ECU specification. For every action specified, a seperate function will be created.

## Specification
ECU configuration files must be in YAML format. The following sections document the properties.

### Top-level
These fields are used for metadata and for package-wide options.

| Field          | Required | Description                                      | 
|:---------------|:---------|:-------------------------------------------------|
| `name`         | Yes      | Will be used as the name of the package.         |
| `version`      | Yes      | Will be used as the version of the package.      |
| `description`  | No       | Will be used as the description of the package.  |
| `kind`         | Yes      | Specifies the kind of the package.               |
| `base`         | No       | Sets the image base for the package.             |
| `contributors` | No       | Lists the contributors to this package.          |
| `environment`  | No       | Lists the environment variables for this package.|
| `dependencies` | No       | Lists the system dependencies for this package.  |

### Install
Coming soon.

### Initialize
Coming soon.

### Entrypoint
Coming soon.

### Actions
Coming soon.

### Types
Coming soon.