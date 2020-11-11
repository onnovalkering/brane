#!/usr/bin/env bash

# Original author: Ivan Eggel (medGIFT group, HES-SO)
# https://github.com/ieggel/process-uc1-integration

set -euo pipefail

function cleanup {
  echo "# Unmounting $target_mnt_dir..."
  umount $target_mnt_dir
}

trap cleanup EXIT
trap cleanup ERR

ssh_server_host="sne-dtn-03.vlan7.uvalight.net"
ssh_server_port_nbr=30909
target_mnt_dir="${HOME}/snedtn"
camelyon17_data_dir="${target_mnt_dir}/camelyon17/working_subset"
intermediate_results_data_dir="${target_mnt_dir}/L3/data/IntermediateResults"

# Create target mount dir if necessary
if [[ -d "$target_mnt_dir" ]]; then
    echo "# Target mount dir ${target_mnt_dir} already exist. Skip creation."
else
    echo "# Creating target mount dir ${target_mnt_dir}."
    mkdir -p $target_mnt_dir
fi

# Add SSH identify file
mkdir -p "${HOME}/.ssh"
echo "${ID_RSA}" | base64 -d > ~/.ssh/id_rsa
chmod 400 ~/.ssh/id_rsa

# Add key to known hosts
ssh-keyscan -p $ssh_server_port_nbr -H $ssh_server_host >> ~/.ssh/known_hosts

# Check if snetdn already mounted
if grep -qs "${target_mnt_dir} " /proc/mounts; then
    echo "# SNEDTN already mounted."
else
    # Mount snetdn via sshfs to target mount dir
    sshfs -o allow_other root@$ssh_server_host:/mnt $target_mnt_dir -p $ssh_server_port_nbr
fi

cd "PROCESS_L3"
CONFIG_FILE="doc/config.cfg"

# Overwrite default configuration
INPUT_FILE_NAME="${PATIENT}"
INPUT_IMAGENET_WEIGHTS="${intermediate_results_data_dir}/Mara/imagenet_models/resnet101_weights_tf.h5"
INPUT_MODEL_WEIGHTS="${intermediate_results_data_dir}/Mara/camnet_models/cam1617_2009/tumor_classifier.h5"
SETTINGS_SOURCE_FLD="${camelyon17_data_dir}/"
SETTINGS_XML_SOURCE_FLD="${camelyon17_data_dir}/lesion_annotations/"
LOAD_PWD="${intermediate_results_data_dir}/Camelyon/all500"

# [input]
sed -i "s|\(file_name *= *\).*|\1$INPUT_FILE_NAME|" $CONFIG_FILE
sed -i "s|\(imagenet_weights *= *\).*|\1$INPUT_IMAGENET_WEIGHTS|" $CONFIG_FILE
sed -i "s|\(model_weights *= *\).*|\1$INPUT_MODEL_WEIGHTS|" $CONFIG_FILE

# [settings]
sed -i "s|\(training_centres *= *\).*|\1$SETTINGS_TRAINING_CENTRES|" $CONFIG_FILE
sed -i "s|\(source_fld *= *\).*|\1$SETTINGS_SOURCE_FLD|" $CONFIG_FILE
sed -i "s|\(xml_source_fld *= *\).*|\1$SETTINGS_XML_SOURCE_FLD|" $CONFIG_FILE
sed -i "s|\(slide_level *= *\).*|\1$SETTINGS_SLIDE_LEVEL|" $CONFIG_FILE
sed -i "s|\(patch_size *= *\).*|\1$SETTINGS_PATCH_SIZE|" $CONFIG_FILE

# [model]
sed -i "s|\(model_type *= *\).*|\1$MODEL_TYPE|" $CONFIG_FILE
sed -i "s|\(loss *= *\).*|\1$MODEL_LOSS|" $CONFIG_FILE
sed -i "s|\(activation *= *\).*|\1$MODEL_ACTIVATION|" $CONFIG_FILE
sed -i "s|\(lr *= *\).*|\1$MODEL_LR|" $CONFIG_FILE
sed -i "s|\(decay *= *\).*|\1$MODEL_DECAY|" $CONFIG_FILE
sed -i "s|\(momentum *= *\).*|\1$MODEL_MOMENTUM|" $CONFIG_FILE
sed -i "s|\(nesterov *= *\).*|\1$MODEL_NESTEROV|" $CONFIG_FILE
sed -i "s|\(batch_size *= *\).*|\1$MODEL_BATCH_SIZE|" $CONFIG_FILE
sed -i "s|\(epochs *= *\).*|\1$MODEL_EPOCHS|" $CONFIG_FILE
sed -i "s|\(verbose *= *\).*|\1$MODEL_VERBOSE|" $CONFIG_FILE

# [load]
sed -i "s|\(PWD *= *\).*|\1$LOAD_PWD|" $CONFIG_FILE

# Run pipeline
python DHeatmap.py &>/dev/null

# Copy output to Brane directory
OUTPUT_DIR="${SETTINGS_OUTPUT_DIR_URL:7}"

cp "results/${PATIENT}.png" $OUTPUT_DIR
cp "results/${PATIENT}_interpolated.png" $OUTPUT_DIR

echo "output:"
echo "  heatmap: file://${OUTPUT_DIR}/${PATIENT}.png"
echo "  heatmap_interpolated: file://${OUTPUT_DIR}/${PATIENT}_interpolated.png"
