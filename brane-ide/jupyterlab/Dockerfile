FROM jupyter/minimal-notebook

COPY kernel /kernel
COPY renderer /renderer

USER root

COPY brane-cli /usr/local/bin

WORKDIR /kernel
RUN python setup.py install \ 
 && python install.py

WORKDIR /renderer
RUN jupyter labextension install /renderer

# Switch back to jovyan to avoid accidental container runs as root
USER $NB_UID
WORKDIR $HOME
