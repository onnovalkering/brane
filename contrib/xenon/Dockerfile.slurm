FROM ubuntu:20.04 as build

ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Europe/Amsterdam

RUN apt-get update && apt-get install -y \
        build-essential \
        uuid-dev \
        libgpgme-dev \
        squashfs-tools \
        libseccomp-dev \
        wget \
        golang-go \
        pkg-config \
        git \
        cryptsetup-bin

ENV VERSION=3.6.2

RUN wget https://github.com/sylabs/singularity/releases/download/v${VERSION}/singularity-${VERSION}.tar.gz && \
    tar -xzf singularity-${VERSION}.tar.gz && \
    cd singularity && \
    ./mconfig -p /usr/local/singularity && \
    make -C ./builddir && \
    make -C ./builddir install

FROM nlesc/xenon-slurm

RUN apt-get update && apt-get install -y \
        ca-certificates \
        libseccomp-dev \
        squashfs-tools \
        sudo \
        tzdata

RUN echo "%xenon  ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers

COPY --from=build /usr/local/singularity /usr/local/singularity

RUN ln /usr/local/singularity/bin/singularity /usr/local/bin/singularity
