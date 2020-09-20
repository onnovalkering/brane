---
layout: default
title: "2. Decode from Base64"
nav_order: 2
parent: Quickstart
---

# 2. Decode from Base64
<span class="label label-red">SYSTEM</span>

In the previous step we've created a function that retreives a README.md file. However, the output appeared to be Base64-encoded. In this step we'll create functions for Base64 encoding/decoding.

A typical implementation in Python, as a command-line application, is:

```python
#!/usr/bin/env python3
import base64
import sys

def decode(s: str) -> str:
  s = s.replace("\n", "")
  b = base64.b64decode(s)
  return b.decode("UTF-8")

def encode(s: str) -> str:
  b = s.encode("UTF-8")
  b = base64.b64encode(b)
  return b.decode("UTF-8")

if __name__ == "__main__":
  command = sys.argv[1]
  argument = sys.argv[2]
  functions = {
    "decode": decode,
    "encode": encode,
  }
  print(functions[command](argument))
```

After saving this code as `run.py` (with execute permission), we can call the functions as follows:
```shell
$ ./run.py encode 'Hello, world!'
$ ./run.py decode 'SGVsbG8sIHdvcmxkIQ=='
```

To make these two functions available to Brane, we have to do the following:

1. Make it explicit for Brane how to run the underlying code of the functions.
2. Make it explicit for Brane wich input parameters to use, and what output to expect.
3. Make small adjustments to the code to make it compatible with Brane.

For the first two tasks we'll write a configuration file, conventionally named `container.yml`.
We start with defining how Brane can run this code. We specify: the list of dependencies that are required (based on the [Ubuntu repository](https://packages.ubuntu.com/focal/)); which files will be used; and what file to consider as entrypoint:

```yaml
dependencies:
  - python3

files:
  - run.py

entrypoint:
  kind: task
  exec: run.py
```

Next, we specify the functions with their input parameters and output:

```yaml
actions:
  'decode':
    command:
      args:
        - decode
    input:
      - type: string
        name: input
    output:
      - type: string
        name: output

  'encode':
    command:
      args:
        - encode
    input:
      - type: string
        name: input
    output:
      - type: string
        name: output
```
Notice that we specify the function name (`decode` / `encode`) as as the only command-line argument, while our Python code currently expects two. This is because inputs will be provided as environment variables to our code. This the most uniform way of passing input, and also supports non command-line applications. The output should be printed as a [YAML mapping](https://yaml.org/spec/1.2/spec.html#mapping) to `stdout`, i.e. as key-value pairs.

Let's adapt our `run.py` code to the above, by making a few small adjustments:

```python
#!/usr/bin/env python3
import base64
import os
import sys
import yaml

def decode(s: str) -> str:
  s = s.replace("\n", "")
  b = base64.b64decode(s)
  return b.decode("UTF-8")

def encode(s: str) -> str:
  b = s.encode("UTF-8")
  b = base64.b64encode(b)
  return b.decode("UTF-8")

if __name__ == "__main__":
  command = sys.argv[1]
  argument = os.environ["INPUT"]
  functions = {
    "decode": decode,
    "encode": encode,
  }
  output = functions[command](argument)
  print(yaml.dump({"output": output}))
```

We, and now also Brane, can call the functions as follows:
```shell
$ INPUT='Hello, world!' ./run.py encode 
$ INPUT='SGVsbG8sIHdvcmxkIQ==' ./run.py decode
```
__TIP__: When it is not possible to directly modify the target code, consider creating a wrapper script.


Before we build our package, we'll make a few last adjustments to our final configuration:

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
  'decode':
    command:
      args:
        - decode
    pattern:
      postfix: decoded
    input:
      - type: string
        name: input
    output:
      - type: string
        name: output

  'encode':
    command:
      args:
        - encode
    pattern:
      postfix: encoded        
    input:
      - type: string
        name: input
    output:
      - type: string
        name: output
```
We've added the `name`, `version` and `kind` metadata properties. Specified an additional dependency: `python3-yaml`. And we've added a call pattern for both functions, more on this later in this quickstart.

See the [ECU](/brane/packages/ecu.html) page for more details about the `container.yml` file, and all the configuration options.

Build the package using the <abbr title="Command-line interface">CLI</abbr>, targeting the configuration file:
```shell
$ brane build container.yml
```

We're done creating functions and building packages. Let's view the packages that we have (Fig. 1):
```
$ brane list
```

<p style="text-align: center">
    <img src="/brane/assets/img/brane-list.png" style="margin-bottom: -35px" alt="list of local packages">
    <br/>
    <sup>Figure 2: list (local) packages</sup>
</p>

[Previous](/brane/quickstart/1-retreive-readme.html){: .btn .btn-outline }
[Next](/brane/quickstart/3-using-the-bakery-repl.html){: .btn .btn-outline }