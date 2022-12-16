---
description: Build packages targeting (RESTful) Web APIs.
---

# Web APIs

Brane can build packages based on [OpenAPI Specification](https://spec.openapis.org/oas/v3.1.0) (OAS). Brane will generate a separate function for every OAS [operation](https://spec.openapis.org/oas/v3.1.0#operation-object). All standard HTTP 1.1 methods are supported.

Web API packages are build using the Brane CLI. Pass an OAS file to the `build` command:

```bash
$ brane build openapi.yaml
```

OAS files may be either in JSON or YAML format.

## Required fields

Brane requires some fields to be present, in addition to the fields already required by OAS.&#x20;

### Metadata

Some top-level OAS fields are used for package metadata.

| Field          | Description                                      |
| -------------- | ------------------------------------------------ |
| `info.title`   | This field specifies the name of the package.    |
| `info.version` | This field specifies the version of the package. |
| `servers`      | This field specifies the URL(s) of the Web APIs. |

For example:

{% code title="openapi.yaml" %}
```yaml
openapi: 3.1.0
info:
  title: httpbin
  version: 0.1.0

servers:
  - url: https://httpbin.org
  
...
```
{% endcode %}

### Operations

Brane requires additional information on operations to generate functions properly.

| Field         | Description                                    |
| ------------- | ---------------------------------------------- |
| `operationId` | This field specifies the name of the function. |

For example:

{% code title="openapi.yaml" %}
```yaml
...

paths:
  /anything:
    get:
      operationId: getAnything
      responses:
        '200':
          description: Anything passed in the request.
          content:
            application/json:
              schema:
                type: object
                required:
                  - data
                properties:
                  data:
                    type: string

...
```
{% endcode %}

## Functions

Coming soon.

### Input & Output

Coming soon.





