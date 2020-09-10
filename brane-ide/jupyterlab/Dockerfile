FROM jupyter/minimal-notebook

USER root

COPY kernel /kernel
COPY renderer /renderer

# Install Brane CLI
RUN mkdir -p $HOME/.local/bin
COPY brane-cli $HOME/.local/bin

# Install kernel and renderer
RUN python /kernel/setup.py install \ 
 && python /kernel/bakery_kernel/install.py \
 && jupyter labextension install /renderer

# Switch back to jovyan
USER $NB_UID
WORKDIR $HOME
