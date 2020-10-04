.PHONY: install

install: install-kernel install-registry install-renderer

install-kernel:
	cd kernel \
 && python setup.py install \
 && python install.py

install-registry:
	jupyter labextension install registry

install-renderer:
	jupyter labextension install renderer

start:
	API_HOST="`hostname`:8080" jupyter lab --ip 0.0.0.0 --LabApp.token=''