---
layout: default
title: 1. Packages
parent: Guide
nav_order: 1
---

# 1. Packages
For now, you only need to have the [Brane CLI](/brane/installation#cli) installed.

In this part of the tutorial we'll create and test packages, one of each supported kind. In the next part we'll use these packages to develop a workflow. Please note that the functionality of these packages is already provided by Brane's [standard library](/brane/references/standard-library). The kind of packages that are supported:

| Kind  | Description                                     | 
|:------|:------------------------------------------------|
| CWL   | Workflows described using the [CWL](https://www.commonwl.org/v1.1/) specification.    |
| DSL   | Scripts written using Brane's domain-specific language: [Bakery](/brane/references/bakery). |
| ECU   | Arbitrary code, containerized using Brane's [ECU](/brane/references/explicit-container-usage) specification. |
| OAS   | Web APIs described using the [OpenAPI](http://spec.openapis.org/oas/v3.0.3) specification. |

For more information about the concepts behind packages, please see the [architecture](/brane/#architecture) section.

## Download a README file
The first package we'll create is an OAS package exposing a single function that retreives a README file from a GitHub repository. We build this package based on the following description (OpenAPI):

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

Packages can be tested locally, by using the `test` command:

![test](/brane/assets/img/test.gif)

## Perform Base64 decoding
As you can see, the README file in the created package is returned as a Base64-encoded string. We need a package that can perform Base64 decoding. We'll use the following CWL document:

```yaml
$base: "https://w3id.org/cwl/cwl#"

$namespaces:
  s: "http://schema.org/"

s:name: 'base64'
s:description: 'Simple Base64 decoding tool.'
s:version: '1.0.0'

cwlVersion: v1.0
class: CommandLineTool
label: base64
baseCommand: "echo"
requirements:
  - class: ShellCommandRequirement

inputs:
  input:
    type: string
    inputBinding:
      position: 1
  pipe:
    type: string
    default: '| base64 -d'
    inputBinding:
      shellQuote: false
      position: 2

outputs:
  output:
    type: stdout
```
Assuming the workflow description is saved as `base64.cwl`, the package can be build using:
```shell
$ brane build base64.cwl
```

## Combine download and decoding
We can create hierachical packages by writing script in the [Bakery](/brane/references/bakery) language. In this example, we combine downloading of the README and the subseqent Base64 decoding into a single function:

```go
---
name: getreadme
version: 1.0.0
---
import "github"
import "b64decode"

owner := ?? as String
repo := ?? as String

readme = getreadme owner repo
return b64decode readme.content
```
Assuming the DSL script is saved as `readme.bk`, the package can be build using:
```shell
$ brane build readme.bk
```

## Split text and count words
... ECU
