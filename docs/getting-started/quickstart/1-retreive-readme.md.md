# 1. Retreive README.md

As a first step, we'll create a function that retreives the README.md file from a GitHub repository. We'll make use of the official [GitHub API](https://docs.github.com/en/rest). When working with such Web APIs, Brane gives us the option of describing the endpoint(s) that we want to call using the [OpenAPI specification](http://spec.openapis.org/oas/v3.0.3) and then [build a package](../../package-builders/web-apis.md) with the corresponding functions based on this specification.

This is a convenient way of saving development time, especially when the OpenAPI specification is already (publicly) available. So, we'll make use of this option, using the specification provided below.

{% code title="github.yml" %}
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
{% endcode %}

It describes a function named `getreadme`. The `getreadme` function has two parameters: `owner` and `repo`. And the output is an object with two properties: `name` and `content`. All of the `string` type.

### Building the package

Save the specification as `github.yml`, and build the package using the CLI (Fig. 1):

```
$ brane build github.yml
```

![Figure 1: building a Web API package from the github.yml file.](<../../.gitbook/assets/Screen Shot 2021-05-03 at 14.04.38.png>)

{% hint style="success" %}
This is the first way of creating custom functions for Brane.
{% endhint %}

### Testing a function

Using the CLI, we can test the `getreadme` function that has been generated for us (Fig. 2):

![](<../../.gitbook/assets/Screen Shot 2021-05-03 at 14.44.04.png>)

It turned out that the value returned by the `getreadme` function is Base64-encoded. This is not the correct format for the word count example. We'll have to create another function to decode it.

{% hint style="info" %}
Adding the `--debug` option results in more detailed (error) messages. For example:\
\
`brane --debug test github`
{% endhint %}
