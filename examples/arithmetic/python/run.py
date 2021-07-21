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
