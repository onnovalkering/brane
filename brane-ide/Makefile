.PHONY: install

install: install-kernel install-extensions

install-kernel:
	cd kernel \
 && python setup.py install \
 && python install.py --sys-prefix

install-extensions: install-js9 install-renderer

install-js9:
	jupyter labextension install extensions/js9

install-renderer:
	jupyter labextension install extensions/renderer

start:
	API_HOST="`hostname`:8080" pipenv shell "jupyter lab --ip 0.0.0.0 --LabApp.token=''"
