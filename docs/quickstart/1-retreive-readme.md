---
layout: default
title: "1. Retreive README.md"
nav_order: 1
parent: Quickstart
---

# 1. Retreive README.md
<span class="label label-blue">Application</span>

As a first step, we'll create a function that retreives the README.md file from a GitHub repository.
We'll make use of the official [GitHub API](https://docs.github.com/en/rest). When working with such Web APIs, Brane gives us the option of describing the endpoint(s) that we want to call using the [OpenAPI specifiction](http://spec.openapis.org/oas/v3.0.3), and then [build a package](/brane/packages/oas.html) based on this specifiction. Which in turn generates the desired function(s) for us. 

This is a convenient way of saving development time, especially when the OpenAPI specification is already (publicly) available. So, we'll make use of this option, using the following specification:

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
      operationId: getreadme
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
Based on this specifiction, a function named `getreadme` will be generated. The function will have two parameters: `owner` and `repo`. And the output will be a [object](/brane/bakery) with two properties: `name` and `content`.
All of the `string` type. The package will be named `github`, with version `1.0.0` (see `info` properties).

## Building an OAS package
Save the specifiction as `github.yml`, and build the package using the <abbr title="Command-line interface">CLI</abbr> (Fig. 1):
```shell
$ brane build github.yml
```

<p style="text-align: center">
    <img src="/brane/assets/img/brane-build-github.png" style="margin-bottom: -35px" alt="package builder output">
    <br/>
    <sup>Figure 1: package builder output.</sup>
</p>

## Testing a function
Using the <abbr title="Command-line interface">CLI</abbr>, we can test the `getreadme` function that has been generated for us (Fig. 2):
```shell
$ brane test github
```

<p style="text-align: center">
    <img src="/brane/assets/img/brane-test-github.png" style="margin-bottom: -35px" alt="package tester output">
    <br/>
    <sup>Figure 2: package tester output</sup>
</p>

The README.md content is Base64-encoded. We need a function that can decode from Base64! 

[Previous](/brane/quickstart/quickstart.html){: .btn .btn-outline }
[Next](/brane/quickstart/2-decode-from-base64.html){: .btn .btn-outline }