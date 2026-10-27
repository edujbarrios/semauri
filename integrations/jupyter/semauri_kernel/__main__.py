"""Launch Semauri through Jupyter's standard kernel transport."""
from ipykernel.kernelapp import IPKernelApp
from .kernel import SemauriKernel

if __name__ == "__main__":
    IPKernelApp.launch_instance(kernel_class=SemauriKernel)
