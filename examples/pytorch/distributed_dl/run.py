#!/usr/bin/env python3
import os
import torch
import torch.distributed as dist
import torch.nn as nn
import torch.optim as optim

from torch.nn.parallel import DistributedDataParallel

# This code (modified to run on CPUs) originates from PyTorch's examples repository (BSD-3-Clause license).

class ToyModel(nn.Module):
    def __init__(self):
        super(ToyModel, self).__init__()
        self.net1 = nn.Linear(10, 10)
        self.relu = nn.ReLU()
        self.net2 = nn.Linear(10, 5)

    def forward(self, x):
        return self.net2(self.relu(self.net1(x)))


def demo_basic():
    print(
        f"[{os.getpid()}] rank = {dist.get_rank()}, "
        + f"world_size = {dist.get_world_size()}"
    )

    model = ToyModel().cpu()
    ddp_model = DistributedDataParallel(model)

    loss_fn = nn.MSELoss()
    optimizer = optim.SGD(ddp_model.parameters(), lr=0.001)

    optimizer.zero_grad()
    outputs = ddp_model(torch.randn(20, 10))
    labels = torch.randn(20, 5)
    loss_fn(outputs, labels).backward()
    optimizer.step()

def spmd_main():
    # These are the parameters used to initialize the process group
    env_dict = {
        key: os.environ[key]
        for key in ("MASTER_ADDR", "MASTER_PORT", "RANK", "WORLD_SIZE")
    }

    print(f"[{os.getpid()}] Initializing process group with: {env_dict}")
    dist.init_process_group(backend="gloo")

    print(
        f"[{os.getpid()}] world_size = {dist.get_world_size()}, "
        + f"rank = {dist.get_rank()}, backend={dist.get_backend()}"
    )

    demo_basic()

    # Tear down the process group
    dist.destroy_process_group()

if __name__ == "__main__":
    spmd_main()
