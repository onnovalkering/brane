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
