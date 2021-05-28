#!/usr/bin/env bash
set -euo pipefail

# Use `launch.py` provided by PyTorch to run the node.
# The environments variables should be passed to the
# container by the container orchestrator in-place.

# The $WORLD_SIZE parameter is always required.
# $RANK and $MASTER_ADDR are required for workers.

python3 -m torch.distributed.launch \
    --nnodes=$WORLD_SIZE \
    --node_rank=${RANK:-0} \
    --master_addr=${MASTER_ADDR:-"0.0.0.0"} \
    --master_port=29500 \
    ./run.py

echo "~~>output: done"
exit 0
