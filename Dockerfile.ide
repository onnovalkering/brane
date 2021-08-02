# 584f43f06586: JupyterLab 3.0.14
FROM jupyter/minimal-notebook:lab-3.0.16

USER root
WORKDIR /

# JuiceFS
ENV VERSION="0.12.1"
ENV RELEASE="juicefs-$VERSION-linux-amd64"

RUN apt-get update && apt-get install -y \
    curl \
 && rm -rf /var/lib/apt/lists/*

RUN curl -LO "https://github.com/juicedata/juicefs/releases/download/v$VERSION/$RELEASE.tar.gz" \
 && tar -xf "$RELEASE.tar.gz" \
 && rm "$RELEASE.tar.gz"

# Install kernel dependencies
RUN pip install \
    filetype==1.0.7 \
    grpcio-tools==1.37.0 \
    grpcio==1.37.0

COPY ./brane-ide/kernels /kernels
COPY ./brane-ide/extensions /extensions

WORKDIR /kernels/bscript
RUN python setup.py install \
 && python install.py

WORKDIR /
RUN jupyter labextension install extensions/renderer

RUN rmdir "$HOME/work"

# Mount DFS before starting Jupyter (start-notebook.sh).
RUN mkdir -p "${HOME}/data" \
 && printf '%s\n' "#!/usr/bin/env bash" >> ./entrypoint.sh \
 && printf '%s\n' "/juicefs mount -d \${BRANE_MOUNT_DFS} ${HOME}/data" >> ./entrypoint.sh \
 && printf '%s\n' "cd ${HOME}" >> ./entrypoint.sh \
 && printf '%s\n' "su jovyan<<'EOF'" >> ./entrypoint.sh \
 && printf '%s\n' "export PATH=\"/opt/conda/bin:$PATH\"" >> ./entrypoint.sh \
 && printf '%s\n' "tini -g -- start-notebook.sh" >> ./entrypoint.sh \
 && printf '%s\n' "EOF" >> ./entrypoint.sh \
 && chmod +x ./entrypoint.sh

ENTRYPOINT [ "./entrypoint.sh" ]
