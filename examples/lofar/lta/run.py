#!/usr/bin/env python3
import base64
import os
import sys
import xmlrpc.client
import yaml


USERNAME = os.environ["USERNAME"]
PASSWORD = os.environ["PASSWORD"]


def get_lta_proxy():
    return xmlrpc.client.ServerProxy(f"https://{USERNAME}:{PASSWORD}@webportal.astron.nl/service-public/xmlrpc")


def files():
    observation_id = os.environ["OBSERVATION_ID"]

    from common.config.Profile import profiles
    profile = profiles.create_profile(USERNAME, PASSWORD)

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


if __name__ == "__main__":
    functions = {
        "files": files,
        "stage": stage,
        "status": status
    }
    
    command = sys.argv[1]
    output = functions[command]()

    print(yaml.dump(output))