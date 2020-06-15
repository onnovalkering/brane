FROM jupyter/minimal-notebook

COPY kernel /kernel

RUN pip install /kernel \
 && python -m bakery_kernel.install

USER root
EXPOSE 8888
