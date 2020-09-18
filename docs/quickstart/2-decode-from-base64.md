---
layout: default
title: "2. Decode from Base64"
nav_order: 2
parent: Quickstart
---

# 2. Decode from Base64
<span class="label label-red">SYSTEM</span>

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

- brane list

[Previous](/brane/quickstart/1-retreive-readme.html){: .btn .btn-outline }
[Next](/brane/quickstart/3-using-the-bakery-repl.html){: .btn .btn-outline }