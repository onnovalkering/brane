.PHONY: install

install: install-kernel install-extensions

install-kernel:
	pipenv shell "cd kernel && python setup.py install && exit" \
 && pipenv shell "cd kernel && python install.py --sys-prefix && exit"

install-extensions: install-js9 install-renderer

install-js9:
	jupyter labextension install extensions/js9

install-registry:
	jupyter labextension install extensions/registry

install-renderer:
	jupyter labextension install extensions/renderer

clear:
	pipenv --rm

start:
	BRANE_API_URL="`hostname`:8080" BRANE_DRV_URL="localhost:50053" pipenv shell "jupyter lab --ip 0.0.0.0 --LabApp.token='' && exit"

generate-grpc:
	pipenv shell "python -m grpc_tools.protoc --proto_path ../brane-drv/proto --python_out=. --grpc_python_out=. ../brane-drv/proto/driver.proto && exit"
