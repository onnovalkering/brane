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