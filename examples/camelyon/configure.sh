#!/usr/bin/env bash

export VAULT_ADDR='http://127.0.0.1:8200'
export VAULT_TOKEN="mytoken"

## This puts default configuration in the vault,
## the SSH key should be added manually.

vault kv put secret/DTN_HOSTNAME value="sne-dtn-03.vlan7.uvalight.net"
vault kv put secret/DTN_PORT value="30909"
vault kv put secret/DTN_MOUNT value="/root/snedtn"
vault kv put secret/CAMELYON17_DATA_DIR value="/camelyon17/working_subset"
vault kv put secret/INTERMEDIATE_RESULTS_DATA_DIR value="/L3/data/IntermediateResults"
vault kv put secret/IMAGENET_WEIGHTS_DIR value="/Mara/imagenet_models"
vault kv put secret/MODEL_WEIGHTS_DIR value="/Mara/camnet_models/cam1617_2009"
vault kv put secret/PWD_DIR value="/Camelyon/all500"
vault kv put secret/H5FILE value="patches.hdf5"

## Uncomment and update with the correct path,
## to automate the DTN_ID_RSA config.

# export DTN_ID_RSA_FILE="~/.ssh/id_rsa"
# vault kv put secret/DTN_ID_RSA value="$(cat $DTN_ID_RSA_FILE | base64 --wrap=0)"
