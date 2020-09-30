#!/usr/bin/env python3
import base64
import os
import os.path
import subprocess
import sys
import tarfile
import xmlrpc.client
import yaml

from os import environ
from os.path import dirname
from urllib.parse import urlparse


def download(): 
    target = os.environ["TARGET_URL"]

    # Write proxy to /opt/wd/proxy
    proxy = base64.b64decode(os.environ["PROXY"]).decode("UTF-8")
    with open("/opt/wd/proxy", "w") as f:
        f.write(proxy)

    # Write copyjob to /opt/wd/copyjob
    surls = [os.environ[f"FILES_{i}_URL"] for i in range(int(os.environ["FILES"]))]
    files = [os.path.join(target, os.path.basename(s)) for s in surls]
    copyjob = '\n'.join([f"{s} {f}" for (s, f) in zip(surls, files)])

    with open("/opt/wd/copyjob", "w") as f:
        f.write(copyjob)

    env = os.environ.copy()
    env["SRM_PATH"] = "/opt/wd/srmclient-2.6.28/usr/share/srm"

    command = [
        "/opt/wd/srmclient-2.6.28/usr/bin/srmcp",
        "-use_urlcopy_script=true",
        "-urlcopy=/opt/wd/lta-url-copy.sh",
        "-server_mode=passive",
        "-x509_user_proxy=/opt/wd/proxy",
        "-copyjobfile=/opt/wd/copyjob"
    ]

    subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True, env=env)

    return {"files": files}


def extract():
    target = urlparse(environ["TARGET_URL"]).path
    files = [urlparse(environ[f"FILES_{i}_URL"]).path for i in range(int(environ["FILES"]))]
    
    directories = []
    for file in files:
        with tarfile.open(file, 'r') as tar:
            tar.extractall(target)

            first_name = tar.getnames()[0]
            directory = first_name if dirname(first_name) == "" else dirname(first_name)
            directories.append(os.path.join(target, directory))

    return {"directories": directories}


def files():
    username = os.environ["USERNAME"]
    password = os.environ["PASSWORD"]
    observation_id = os.environ["OBSERVATION_ID"]

    from common.config.Profile import profiles
    profile = profiles.create_profile(username, password)

    from awlofar.database.Context import context
    from awlofar.main.aweimports import CorrelatedDataProduct as cdp, FileObject, BeamFormedDataProduct, Observation
    from common.database.Database import database
    if not database.connected():
        database.connect()

    query_observations = Observation.observationId == observation_id
    files = []
    for observation in query_observations:
        dataproduct_query = cdp.observations.contains(observation)
        dataproduct_query &= cdp.isValid == 1
        
        for dataproduct in dataproduct_query:
            fileobject = ((FileObject.data_object == dataproduct) & (FileObject.isValid > 0)).max('creation_date')
            if fileobject:
                files.append(fileobject.URI)

    return {"files": files}


def stage():
    files_n = int(os.environ["FILES"])
    files = [os.environ[f"FILES_{i}_URL"] for i in range(files_n)]

    lta_proxy = get_lta_proxy()
    request_id = lta_proxy.LtaStager.add_getid(files)

    return {'request_id': request_id}


def status():
    request_id = int(os.environ["REQUEST_ID"])

    lta_proxy = get_lta_proxy()
    try:
        status = lta_proxy.LtaStager.getstatus(int(request_id))
    except:
        status = "unkown" 

    return {"status": status}


def get_lta_proxy():
    username = os.environ["USERNAME"]
    password = os.environ["PASSWORD"]

    return xmlrpc.client.ServerProxy(f"https://{username}:{password}@webportal.astron.nl/service-public/xmlrpc")


if __name__ == "__main__":
    functions = {
        "download": download,
        "extract": extract,
        "files": files,
        "stage": stage,
        "status": status
    }
    
    command = sys.argv[1]
    output = functions[command]()

    print(yaml.dump(output))