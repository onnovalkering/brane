#!/usr/bin/env python3
import base64
import os
import yaml


if __name__ == "__main__":
    secret = os.environ['SECRET']
    print(yaml.dump({"secret": secret}))
