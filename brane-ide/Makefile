.PHONY: install

install: install-kernel install-extensions

install-kernel:
	cd kernel \
	pipenv run -- python setup.py install \
 && pipenv run -- python install.py --sys-prefix

install-extensions: install-js9 install-renderer

install-js9:
	pipenv run -- jupyter labextension install extensions/js9

install-registry:
	pipenv run -- jupyter labextension install extensions/registry

install-renderer:
	pipenv run -- jupyter labextension install extensions/renderer

clear:
	pipenv --rm

start:
	BRANE_API_URL="`hostname`:8080" BRANE_DRV_URL="localhost:50053" pipenv run -- jupyter lab --ip 0.0.0.0 --LabApp.token=''

generate-grpc:
	pipenv run -- python -m grpc_tools.protoc --proto_path ../brane-drv/proto --python_out=. --grpc_python_out=. ../brane-drv/proto/driver.proto
