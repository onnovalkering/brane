FROM jupyter/minimal-notebook

COPY kernel /kernel
COPY renderer /renderer

USER root

ADD https://github.com/onnovalkering/brane/releases/download/v0.1.0/brane-cli /usr/local/bin
RUN chmod +x /usr/local/bin/brane-cli
RUN brane-cli -s login http://brane-api:8080 --username jovyan

WORKDIR /kernel
RUN python setup.py install \ 
 && python install.py

WORKDIR /renderer
RUN jupyter labextension install /renderer

# Switch back to jovyan to avoid accidental container runs as root
USER $NB_UID
WORKDIR $HOME
