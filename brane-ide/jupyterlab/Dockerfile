FROM jupyter/minimal-notebook

COPY kernel /kernel

RUN pip install /kernel \
 && python -m bakery_kernel.install

EXPOSE 8888
