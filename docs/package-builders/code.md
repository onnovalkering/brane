---
description: Build packages based on arbitrary code.
---

# Code

To build a package based on arbitrary code, you must explicitly describe, in a configuration file, how Brane can execute the code. The configuration file needs to be in the YAML format, follow the ECU specification on this page, and is conventionally called `container.yml`. Then, build the package using the command:

```bash
$ brane build container.yml
```

## Specification

Brane's specification to explicitly describe arbitrary code is called _explicit container usage_ (ECU).

The following sections document the properties of an ECU document.

### Root-level

These root-level fields are for metadata and package-wide options.

| Field          | Required | Description                                               |
| -------------- | -------- | --------------------------------------------------------- |
| `name`         | Yes      | Will be used as the name of the package.                  |
| `version`      | Yes      | Will be used as the version of the package.               |
| `description`  | No       | Will be used as the description of the package.           |
| `kind`         | Yes      | Specifies the kind of package.                            |
| `base`         | No       | Sets the image base for the package.                      |
| `contributors` | No       | Lists the contributors to this package.                   |
| `environment`  | No       | Lists the environment variables for this package.         |
| `dependencies` | No       | Lists the system dependencies for this package.           |
| `files`        | No       | Lists the files that need to be included in this package. |

{% hint style="warning" %}
Currently, there is only one kind of package: **compute**.
{% endhint %}

{% hint style="warning" %}
Only the [ubuntu](https://hub.docker.com/\_/ubuntu) and [alpine](https://hub.docker.com/\_/alpine) **** images are supported as base images, the default is **ubuntu:20.04**.&#x20;
{% endhint %}

### Install

You can use the `install` field to specify commands that will be executed as part of the package setup. The package builder will convert each line into a `RUN` statement in the resulting `Dockerfile`.

{% tabs %}
{% tab title="container.yml" %}
```yaml
install:
  - pip install requests
  - ./install.sh
```
{% endtab %}

{% tab title="Dockerfile" %}
```yaml
RUN pip install requests
RUN ./install.sh
```
{% endtab %}
{% endtabs %}

### Initialize

Similarly to the `install` field, the `initialize` field is a list of commands that will be executed as part of the package setup. However, these commands are run every time the package is invoked.

### Entrypoint

The `entrypoint` specifies what file or command is run once the package is invoked.

{% tabs %}
{% tab title="container.yml" %}
```yaml
entrypoint:
  kind: task
  exec: ./run.sh
```
{% endtab %}
{% endtabs %}

{% hint style="warning" %}
Make sure the entrypoint file has **execute** permission (i.e.,`chmod +x run.sh`).
{% endhint %}

### Actions

One or more `actions` can be specified to indicate different ways of invoking the package. Each action (i.e., function) needs to explicitly specify the input and output. These variables are passed as environment variables to the program specified under the `entrypoint` field (see the [input](code.md#input) section).

{% tabs %}
{% tab title="container.yml" %}
```yaml
actions:
  add:
    command:
      args:
        - add

    input:
      - name: a
        type: integer
      - name: b
        type: integer

    output:
      - name: c
        type: integer
```
{% endtab %}
{% endtabs %}

{% hint style="info" %}
There can only be one output value.
{% endhint %}

The type of values that are supported:

| Name    | Type                          | Description                          |
| ------- | ----------------------------- | ------------------------------------ |
| Boolean | `boolean`                     | A boolean value: true or false.      |
| Integer | `integer`                     | A signed integer (64bit).            |
| Real    | `real`                        | A signed float/double (64bit).       |
| String  | `string`                      | A UTF-8 string.                      |
| Array   | `integer[]`, `string[]`,  ... | An array of values of the same type. |

## Input

As mentioned before, input arguments are passed as environment variables. This is straightforward for single values: there is direct mapping based on the name of the input argument (see the [example](code.md#example) below).&#x20;

### Arrays

For input arrays, there is a special mapping with multiple environment variables. A variable with the same name as the input argument contains the number of elements in the array. Individual elements of the array are made available as environment variables with the name: `{array}_{i}` where `{array}` is the name of the input argument, and `{i}` is the index of the element, e.g., `ARRAY_0`, `ARRAY_1`, et cetera.&#x20;

{% tabs %}
{% tab title="Python" %}
```python
from os import environ

# We can construct a string[] using environment variables directly.
[environ[f"ARRAY_{i}"] for i in range(int(environ["ARRAY"]))]

# For integer[] and float[] we have to cast each element using `int` and `float`.
[int(environ[f"ARRAY_{i}"]) for i in range(int(environ["ARRAY"]))]
[float(environ[f"ARRAY_{i}"]) for i in range(int(environ["ARRAY"]))]

# For boolean[] we need a helper method, the `bool` built-in won't work.
bool = lambda b: b.lower() == "true"
[bool(environ[f"ARRAY_{i}"]) for i in range(int(environ["ARRAY"]))]
```
{% endtab %}

{% tab title="Node.js" %}
```javascript
const env = process.env;

// We can construct a string[] using environment variables directly.
Array.from({length: env["ARRAY"]}, (_, i) => env[`ARRAY_${i}`]);

// For integer[] and float[] we have to cast each element using `Number`.
Array.from({length: env["ARRAY"]}, (_, i) => Number(env[`ARRAY_${i}`]));

// For boolean[] we need a helper method, the `Boolean` built-in won't work.
const Boolean = b => b.toLowerCase() === "true";
Array.from({length: env["ARRAY"]}, (_, i) => Boolean(env[`ARRAY_${i}`]));
```
{% endtab %}

{% tab title="Bash" %}
```bash
#!/usr/bin/env bash
set -euo pipefail

# In Bash, we simply store array elements as strings.
declare -a array
for i in $(seq 0 $(($ARRAY-1)))
do
    element="ARRAY_$i"
    array[$i]=${!element}
done

# Alternativly, we can directly access elements by name.
echo $ARRAY_0
echo $ARRAY_1
echo $ARRAY_2
```
{% endtab %}
{% endtabs %}

## Output

Brane captures the `stdout` as output, and expects it to be in YAML format. Consider an output variable `c` of type `integer` (see the container.yml example below). The `stdout` might be:

```css
c: 1
```

### Capture modes

Since your application might also print log statements to `stdout`, Brane supports different  capture modes: `complete` (default), `marked`, and `prefixed`. You specify the capture mode per action under the `command` property:

```yaml
actions:
  add:
    command:
      capture: complete
      args:
        - add

    input:
      - name: a
        type: integer
      - name: b
        type: integer

    output:
      - name: c
        type: integer
```

#### Complete

As the name implies, this mode captures the complete `stdout` as output.

#### Marked

With the `marked` mode, only a proportion of the `stdout` is captured that is delimited by two start and end markers: `--> START CAPTURE` and `--> END CAPTURE`. For example:

```css
[DEBUG] application is starting
--> START CAPTURE
c: 1

--> END CAPTURE
[DEBUG] application is done
```

#### Prefixed

It might be that multiple threads simultaneously write to `stdout`. In this case the `marked` capture mode might not work and the `prefixed` capture mode must be used. Add a `~~>` prefix as follows:

```css
[DEBUG] application is starting
~~> c: 1
[DEBUG] application is done
```

## Example

The following two files are for a package with basic (integer) arithmetic functions.

{% tabs %}
{% tab title="run.sh" %}
```bash
#!/usr/bin/env python3
import math
import os
import sys
import yaml

def add(a: int, b: int) -> int:
  return a + b

def substract(a: int, b: int) -> int:
  return a - b

def multiply(a: int, b: int) -> int:
  return a * b

def divide(a: int, b: int) -> int:
  return math.floor(a / b)

if __name__ == "__main__":
  functions = {
    "add": add,
    "substract": substract,
    "multiply": multiply,
    "divide": divide,
  }

  operation = sys.argv[1]
  a = int(os.environ["A"])
  b = int(os.environ["B"])

  output = functions[operation](a, b)
  print(yaml.dump({"c": output}))
```
{% endtab %}

{% tab title="container.yml" %}
```yaml
name: arithmetic
version: 1.0.0
kind: compute
base: alpine

entrypoint:
  kind: task
  exec: run.py

dependencies:
  - python3
  - py3-pip

install:
  - pip3 install pyyaml

files:
  - run.py

actions:
  add:
    command:
      args:
        - add

    pattern:
      prefix: "add"
      infix:
        - "to"

    input:
      - name: a
        type: integer
      - name: b
        type: integer

    output:
      - name: c
        type: integer

  substract:
    command:
      args:
        - substract

    pattern:
      prefix: "substract"
      infix:
        - "from"

    input:
      - name: a
        type: integer
      - name: b
        type: integer

    output:
      - name: c
        type: integer

  multiply:
    command:
      args:
        - multiply

    pattern:
      prefix: "multiply"
      infix:
        - "by"

    input:
      - name: a
        type: integer
      - name: b
        type: integer

    output:
      - name: c
        type: integer

  divide:
    command:
      args:
        - divide

    pattern:
      prefix: "divide"
      infix:
        - "by"

    input:
      - name: a
        type: integer
      - name: b
        type: integer

    output:
      - name: c
        type: integer
```
{% endtab %}
{% endtabs %}

Variations of this package in different languages are available in the [examples](https://github.com/onnovalkering/brane/tree/master/examples/arithmetic) directory (GitHub).
