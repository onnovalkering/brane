---
layout: default
title: Quickstart
nav_order: 4
description: "Quickstart"
permalink: /quickstart
---

# Quickstart
In this hands-on quickstart you'll become aquinted with basics of the Brane framework.

To install Brane, please see the [installation](/brane/installation) page.

## 1. Packages
In the first part we'll create and test packages, one of each supported kind. In the next part we'll use these packages to construct workflows. Note that some functionality of the packages created here is already provided by Brane's [standard library](/brane/standard-library). The four supported kind of packages are:

| Kind  | Description                                     | 
|:------|:------------------------------------------------|
| CWL   | Workflows described using the [CWL](https://www.commonwl.org/v1.1/) specification.    |
| DSL   | Scripts written in Brane's domain-specific language: [Bakery](/brane/bakery). |
| ECU   | Arbitrary code, containerized using Brane's [ECU](/brane/references/explicit-container-usage) specification. |
| OAS   | Web APIs described using the [OpenAPI](http://spec.openapis.org/oas/v3.0.3) specification. |

For more information about the concepts behind packages, please see the [architecture](/brane/#architecture) section.

The source code of the packages can be found in the [examples](https://github.com/onnovalkering/brane/tree/master/examples/wordcount) directory of the GitHub repository.

### Download a README file
The first package we'll create is an OAS package exposing a single function that retreives a README file from a GitHub repository. This package is build based on the following description (OpenAPI):

```yaml
openapi: 3.0.0
info:
  title: GitHub
  version: 1.0.0

servers:
  - url: https://api.github.com

paths:
  '/repos/{owner}/{repo}/readme':
    get:
      operationId: getReadme
      parameters:
        - name: owner
          in: path
          required: true
          schema:
            type: string
        - name: repo
          in: path
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Information about the README
          content:
            application/json:
              schema:
                type: object
                required:
                  - name
                  - content
                properties:
                  name:
                    type: string
                  content:
                    type: string
```
Assuming the API description is saved as `github.yml`, the package can be build using:
```shell
$ brane build github.yml
```

Packages can be tested locally using the `test` command:

![test](/brane/assets/img/test.gif)

### Perform Base64 decoding
The retreived README files are Base64-encoded. We need a package than can perform Base64 decoding so we can read them. We'll make an ECU package based on the following two files:
```yaml
name: base64
version: 1.0.0
kind: compute

dependencies:
  - python3
  - python3-yaml

files:
  - run.py

entrypoint:
  kind: task
  exec: run.py

actions:
  'b64decode':
    command:
      args:
        - decode
    pattern:
      prefix: b64decode
    input:
      - type: string
        name: input
    output:
      - type: string
        name: output

  'b64encode':
    command:
      args:
        - encode
    pattern:
      prefix: b64encode
    input:
      - type: string
        name: input
    output:
      - type: string
        name: output
```
```python
#!/usr/bin/env python3
import base64
import os
import sys
import yaml

command = sys.argv[1]
argument = os.environ['INPUT']


functions = {
    "encode": lambda x: base64.b64encode(x.encode("UTF-8")).decode("UTF-8"),
    "decode": lambda x: base64.b64decode(x).decode("UTF-8"),
}

if __name__ == "__main__":
    argument = argument.replace("\n", "")
    output = functions[command](argument)

    print(yaml.dump({"output": output}))
```

Conventionally, ECU descriptions are saved as `container.yml`. Thus we build the package using:
```shell
$ brane build container.yml
```

## Combine download and decoding
We can create hierachical packages by writing script in the [Bakery](/brane/references/bakery) language. In this example, we combine downloading of the README and the subseqent Base64 decoding into a single function:

```go
---
name: getreadme
version: 1.0.0
---
import "github"
import "base64"

owner := ?? as String
repo := ?? as String

readme := getreadme owner repo
return b64decode readme.content
```
Assuming the DSL script is saved as `readme.bk`, the package can be build using:
```shell
$ brane build readme.bk
```

### Cowsay
The last package is a simple CWL package that calls `cowsay`:

```yaml
$base: "https://w3id.org/cwl/cwl#"

$namespaces:
  s: "http://schema.org/"

s:name: 'cowsay'
s:version: '1.0.0'

cwlVersion: v1.0
class: CommandLineTool
label: cowsay
baseCommand: /usr/games/cowsay
hints:
  DockerRequirement:
    dockerPull: chuanwen/cowsay

inputs:
  input:
    type: string
    inputBinding:
      position: 1

outputs:
  output:
    type: stdout
```
We build this package with:
```shell
$ brane build cowsay.cwl
```

### Publishing packages
We can publish packages by pushing them to the registry of a Brane instance. But first we have to pair the CLI to the brane instance. This is done by logging in (use the hostname of your instance):
```shell
$ brane login localhost:8080 --username joyvan
```

After logging in, we can push the packages:
```shell
$ brane push github 1.0.0
$ brane push base64 1.0.0
$ brane push getreadme 1.0.0
$ brane push cowsay 1.0.0
```
Now we can use these packages in our [workflows](/brane/guide/workflows).

## 2. Workflows
Now we can start writing workflows. Consider the following workflow:
```go
import "cowsay"
import "getreadme"
import "text"

readme := getreadme "onnovalkering" "brane"
words := split readme

message := "This readme contains " + words.length + " words."
cowsay message
```

Writing this workflow is very easy and we don't have to deal with any technicalities, under the hood Brane will take care of:

- calling a remote web service (`getreadme`);
- executing containerized code (`b64decode`);
- executing a CWL workflow (`cowsay`).
- basic string manipulation (`split`; from the [std](/brane/references/standard-library.html))

![test](/brane/assets/img/jupyter.png)

