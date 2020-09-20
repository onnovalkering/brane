# 612aa5710bf9: updated Sept 2020
FROM jupyter/minimal-notebook:612aa5710bf9

COPY kernel /kernel
COPY registry /registry
COPY renderer /renderer

USER root

ADD https://github.com/onnovalkering/brane/releases/download/v0.1.0/brane-linux /usr/local/bin/brane-cli
RUN chmod +x /usr/local/bin/brane-cli
RUN brane-cli -s login http://brane-api:8080 --username jovyan

WORKDIR /kernel
RUN python setup.py install \ 
 && python install.py

RUN jupyter labextension install /registry
RUN jupyter labextension install /renderer

# Switch back to jovyan to avoid accidental container runs as root
USER $NB_UID
WORKDIR $HOME
