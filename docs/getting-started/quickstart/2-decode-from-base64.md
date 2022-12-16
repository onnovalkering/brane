# 2. Decode from Base64

In the previous step we created a function that retreives a README.md file. However, the output appeared to be Base64-encoded. In this step we add the missing Base64 encoding and decoding functions.

A typical implementation in Python is:

{% code title="run.py" %}
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
{% endcode %}

After saving this code as `run.py`, we can call the functions as follows:

```bash
$ chmod +x ./run.py
$ ./run.py encode 'Hello, world!'
$ ./run.py decode 'SGVsbG8sIHdvcmxkIQ=='
```

### Build the package

To make the encoding/decoding functions available to Brane, we have to perform three tasks:

1. Make it explicit for Brane how to run the underlying code of the functions.
2. Make it explicit for Brane which input parameters to use, and what output to expect.
3. Make small adjustments to the code to make it compatible with Brane.

To make explicit for Brane how to run the encoding/decoding functions and describe the input/output parameters, we write a configuration file, conventionally named `container.yml`. In this file we describe how Brane can run this code, we specify: the list of system dependencies that are required (based on the [Ubuntu repository](https://packages.ubuntu.com/focal/)); which files will be used; and what file to consider the entrypoint:

{% code title="container.yml" %}
```yaml
dependencies:
  - python3

files:
  - run.py

entrypoint:
  kind: task
  exec: run.py
```
{% endcode %}

Next, we specify the functions with their input/output parameters:

{% code title="container.yml" %}
```yaml
...

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
{% endcode %}

We specify the function name (`decode` / `encode`) as as the only command-line argument, while our Python code currently expects two. This is because inputs will be provided through environment variables to our code. This the most uniform way of passing input, and also supports non-command-line applications. The output should be printed as a [YAML mapping](https://yaml.org/spec/1.2/spec.html#mapping) to `stdout`, i.e. as key-value pairs.

Let's adapt our `run.py` code to the above, by making a few small adjustments:

{% code title="run.py" %}
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
{% endcode %}

We, and now also Brane, can call the functions as follows (requires the `pyyaml` package):

```bash
$ INPUT='Hello, world!' ./run.py encode 
$ INPUT='SGVsbG8sIHdvcmxkIQ==' ./run.py decode
```

{% hint style="info" %}
When it is not possible to modify the target source code, consider creating a wrapper script.
{% endhint %}

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

We've added the `name`, `version` and `kind` metadata properties. Specified an additional dependency: `python3-yaml`.  See the [container package builder](../../package-builders/code.md) page for more details about the `container.yml` file, and all the configuration options. Build the package using the CLI, targeting the configuration file:

```
$ brane build container.yml
```

{% hint style="success" %}
This is the second way of creating custom functions for Brane.
{% endhint %}

### View local packages

Let's view the packages that we have created so far (Fig. 1):

```
$ brane list
```

![Figure 1: the Brane CLI can show the locally available packages.](<../../.gitbook/assets/Screen Shot 2021-05-03 at 14.48.28.png>)

\
