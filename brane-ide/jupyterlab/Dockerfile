FROM jupyter/minimal-notebook

# Install Brane CLI
RUN mkdir -p $HOME/.local/bin
COPY brane $HOME/.local/bin

USER root

# Install Bakery kernel
COPY kernel /kernel
WORKDIR /kernel
RUN python setup.py install && python bakery_kernel/install.py

# Install Brane renderer
COPY renderer /renderer
WORKDIR /renderer
RUN jupyter labextension install .

USER $NB_UID
WORKDIR $HOME
