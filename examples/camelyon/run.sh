#!/usr/bin/env bash
set -euo pipefail

# Original author: Ivan Eggel (medGIFT group, HES-SO)
# https://github.com/ieggel/process-uc1-integration

function cleanup {
  echo "# Unmounting $DTN_MOUNT..."
  umount $DTN_MOUNT
}

trap cleanup EXIT
trap cleanup ERR

CAMELYON17_DATA_DIR="${DTN_MOUNT}${CAMELYON17_DATA_DIR}"
INTERMEDIATE_RESULTS_DATA_DIR="${DTN_MOUNT}${INTERMEDIATE_RESULTS_DATA_DIR}"

# Create target mount dir if necessary
if [[ -d "$DTN_MOUNT" ]]; then
    echo "# Target mount dir ${DTN_MOUNT} already exist. Skip creation."
else
    echo "# Creating target mount dir ${DTN_MOUNT}."
    mkdir -p $DTN_MOUNT
fi

# Add SSH identify file
mkdir -p "${HOME}/.ssh"
echo "${DTN_ID_RSA}" | base64 -d > ~/.ssh/id_rsa
chmod 400 ~/.ssh/id_rsa

# Add key to known hosts
ssh-keyscan -p $DTN_PORT -H $DTN_HOSTNAME >> ~/.ssh/known_hosts

# Check if snetdn already mounted
if grep -qs "${DTN_MOUNT} " /proc/mounts; then
    echo "# SNEDTN already mounted."
else
    # Mount snetdn via sshfs to target mount dir
    sshfs -o allow_other root@$DTN_HOSTNAME:/mnt $DTN_MOUNT -p $DTN_PORT
fi

cd "PROCESS_L3"

CONFIG_FILE="doc/config.cfg"
INPUT_IMAGENET_WEIGHTS="${INTERMEDIATE_RESULTS_DATA_DIR}${IMAGENET_WEIGHTS_DIR}/${INPUT_IMAGENET_WEIGHTS}"
INPUT_MODEL_WEIGHTS="${INTERMEDIATE_RESULTS_DATA_DIR}${MODEL_WEIGHTS_DIR}/${INPUT_MODEL_WEIGHTS}"
SETTINGS_SOURCE_FLD="${CAMELYON17_DATA_DIR}/"
SETTINGS_XML_SOURCE_FLD="${CAMELYON17_DATA_DIR}/lesion_annotations/"
LOAD_PWD="${INTERMEDIATE_RESULTS_DATA_DIR}${PWD_DIR}"
LOAD_H5FILE="${H5FILE}"

# [settings]
sed -i "s|\(training_centres *= *\).*|\1$SETTINGS_TRAINING_CENTRES|" $CONFIG_FILE
sed -i "s|\(source_fld *= *\).*|\1$SETTINGS_SOURCE_FLD|" $CONFIG_FILE
sed -i "s|\(xml_source_fld *= *\).*|\1$SETTINGS_XML_SOURCE_FLD|" $CONFIG_FILE
sed -i "s|\(slide_level *= *\).*|\1$SETTINGS_SLIDE_LEVEL|" $CONFIG_FILE
sed -i "s|\(patch_size *= *\).*|\1$SETTINGS_PATCH_SIZE|" $CONFIG_FILE
sed -i "s|\(n_samples *= *\).*|\1$SETTINGS_N_SAMPLES|" $CONFIG_FILE

# [input]
sed -i "s|\(file_name *= *\).*|\1$INPUT_FILE_NAME|" $CONFIG_FILE
sed -i "s|\(imagenet_weights *= *\).*|\1$INPUT_IMAGENET_WEIGHTS|" $CONFIG_FILE
sed -i "s|\(model_weights *= *\).*|\1$INPUT_MODEL_WEIGHTS|" $CONFIG_FILE
sed -i "s|\(interpret *= *\).*|\1$INPUT_INTERPRET|" $CONFIG_FILE
sed -i "s|\(i_n_samples *= *\).*|\1$INPUT_N_SAMPLES|" $CONFIG_FILE
sed -ie '0,/i_n_samples/ s|i_n_samples|n_samples|' $CONFIG_FILE # Remove prefix on first 'n_samples'

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
sed -i "s|\(h5file *= *\).*|\1$LOAD_H5FILE|" $CONFIG_FILE

# Copy output to Brane directory
OUTPUT_DIR="${SETTINGS_OUTPUT_DIR_URL:7}"

# Run pipeline
LOG_FILE="${OUTPUT_DIR}/dheatmap_logs.txt"
python DHeatmap.py &>"${LOG_FILE}"

# Copy results to persistent storage (accessible by Brane)
cp -r "results" $OUTPUT_DIR

# Parse some values
export N_MODEL=$(awk '/^Distributing inference/ {print $4; exit}' ${LOG_FILE})
export N_PATCHES=$(awk '/^Number of patches/ {print $5; exit}' ${LOG_FILE})
export ELAPSED_TIME=$(awk '/^Elapsed time/ {print $3; exit}' ${LOG_FILE})

echo "output:"
echo "  logs: file://${LOG_FILE}"
echo "  elapsed_time: $ELAPSED_TIME"
echo "  n_model: $N_MODEL"
echo "  n_patches: $N_PATCHES"
echo "  heatmap: file://${OUTPUT_DIR}/results/${INPUT_FILE_NAME}.png"
echo "  heatmap_interpolated: file://${OUTPUT_DIR}/results/${INPUT_FILE_NAME}_interpolated.png"
echo "  interpretability:"
FILES="${OUTPUT_DIR}/results/interpretability/${INPUT_FILE_NAME}/*.png"
for filename in $FILES; do
    echo "    - file://${filename}"
done
