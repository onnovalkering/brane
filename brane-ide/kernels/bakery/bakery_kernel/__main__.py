from ipykernel.kernelapp import IPKernelApp
from . import BakeryKernel

IPKernelApp.launch_instance(kernel_class=BakeryKernel)
