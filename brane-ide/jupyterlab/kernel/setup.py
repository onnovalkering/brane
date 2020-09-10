from bakery_kernel import __version__
from distutils.core import setup
from setuptools import find_packages

setup(
    name='bakery_kernel',
    version=__version__,
    packages=find_packages(include=['bakery_kernel']),
)
