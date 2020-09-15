#!/usr/bin/env python3
import base64
import os
import os.path
import subprocess
import sys
import xmlrpc.client
import yaml

def download(): 
    target = os.environ["TARGET_URL"]

    # Write proxy to /opt/wd/proxy
    proxy = base64.b64decode(os.environ["PROXY"]).decode("UTF-8")
    with open("/opt/wd/proxy", "w") as f:
        f.write(proxy)

    # Write copyjob to /opt/wd/copyjob
    surls = [os.environ[f"SURLS_{i}"] for i in range(int(os.environ["SURLS"]))]
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
    surls = []
    for observation in query_observations:
        dataproduct_query = cdp.observations.contains(observation)
        dataproduct_query &= cdp.isValid == 1
        
        for dataproduct in dataproduct_query:
            fileobject = ((FileObject.data_object == dataproduct) & (FileObject.isValid > 0)).max('creation_date')
            if fileobject:
                surls.append(fileobject.URI)

    return {"surls": surls}


def stage():
    surls_n = int(os.environ["SURLS"])
    surls = [os.environ[f"SURLS_{i}"] for i in range(surls_n)]

    lta_proxy = get_lta_proxy()
    request_id = lta_proxy.LtaStager.add_getid(surls)

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
        "files": files,
        "stage": stage,
        "status": status
    }
    
    command = sys.argv[1]
    output = functions[command]()

    print(yaml.dump(output))