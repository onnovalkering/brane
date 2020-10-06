#!/usr/bin/env python3
import sys
import yaml

from os import environ
from os.path import basename, join
from shutil import copyfile
from urllib.parse import urlparse

def load():
    target = urlparse(environ["TARGET_URL"]).path
    sources = [
        "/opt/wd/L570745_SB000_uv_first10.MS.tar",
        "/opt/wd/L570745_SB001_uv_first10.MS.tar"
    ]
    
    files = []
    for source in sources:
        file = join(target, basename(source))
        files.append(file)

        # copyfile(source, file)

    return {"files": files}


if __name__ == "__main__":
    functions = {
        "load": load,
    }
    
    command = sys.argv[1]
    output = functions[command]()

    print(yaml.dump(output))