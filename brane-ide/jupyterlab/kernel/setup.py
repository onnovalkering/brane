from distutils.core import setup
from bakery_kernel import __version__

setup(
    name='bakery_kernel',
    version=__version__,
    packages=['bakery_kernel'],
    description='A Bakery kernel for Jupyter',
    long_description='',
    author='Onno Valkering',
    url='https://github.com/onnovalkering/brane',
    classifiers=[],
)
