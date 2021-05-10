# https://docs.datastax.com/en/developer/cpp-driver/2.16

version = 2.16.0

all: install-dependencies build cleanup

install-dependencies:
	apt-get update && apt-get install -y \
		build-essential \
		cmake \
		git \
		libkrb5-dev \
		libssl-dev \
		zlib1g-dev \
		libuv1 \
		libuv1-dev

build: checkout
	cd cpp-driver \
	&& mkdir -p build \
	&& cd build \
	&& cmake .. \
	&& make \
	&& make install

checkout:
	if [ ! -d "./cpp-driver" ]; then \
		git clone --depth 1 --branch ${version} https://github.com/datastax/cpp-driver.git; \
	fi

cleanup:
	rm -rf cpp-driver
