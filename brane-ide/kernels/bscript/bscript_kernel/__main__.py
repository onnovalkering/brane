from ipykernel.kernelapp import IPKernelApp
from . import BraneScriptKernel

IPKernelApp.launch_instance(kernel_class=BraneScriptKernel)
