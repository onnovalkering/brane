---
layout: default
title: OAS
parent: Packages
nav_order: 4
---

# OpenAPI
A package can be built based on a [OpenAPI](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.3.md) specifiction.

```shell
$ brane build web-api.yaml
```

Point the `build` command of the Brane CLI to a file containing the OpenAPI specification. For every action to an endpoint, a seperate function will be created.

### Metadata
Relevant metadata can be added to the CWL document

| Field       | Required | Description                                 | 
|:------------|:---------|:--------------------------------------------|
| `title`     | Yes      | Will be used as the name of the package.    |
| `version`   | Yes      | Will be used as the version of the package. |

These fields are part of the OpenAPI specification, and specified under the `info` field:

```yaml
openapi: 3.0.0
info:
    title: myapi
    version: 1.0.0
```