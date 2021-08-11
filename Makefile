build: build-binaries build-services

# Build release versions of the binaries.
build-binaries: \
	build-cli \
	build-let

build-cli:
	cargo build --release --package brane-cli

build-let:
	rustup target add x86_64-unknown-linux-musl
	cargo build --release --package brane-let --target x86_64-unknown-linux-musl

# build release versions of the serivces.
build-services: \
	build-api-image \
	build-clb-image \
	build-drv-image \
	build-job-image \
	build-log-image \
	build-plr-image

build-api-image:
	docker build --load -t brane_brane-api -f Dockerfile.api .

build-clb-image:
	docker build --load -t brane_brane-clb -f Dockerfile.clb .

build-drv-image:
	docker build --load -t brane_brane-drv -f Dockerfile.drv .

build-job-image:
	docker build --load -t brane_brane-job -f Dockerfile.job .

build-log-image:
	docker build --load -t brane_brane-log -f Dockerfile.log .

build-plr-image:
	docker build --load -t brane_brane-plr -f Dockerfile.plr .

# Development setup
start-instance: \
	ensure-configuration \
	create-kind-network \
	start-services \
	format-dfs \
	start-brane

stop-instance: \
	stop-ide \
	stop-brane \
	stop-services

start-services:
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-svc.yml up -d

stop-services:
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-svc.yml down

restart-services: stop-services start-services

start-brane:
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-brn.yml up -d

stop-brane:
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-brn.yml down

start-ide:
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-ide.yml up -d

stop-ide:
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-ide.yml down

# Configuration

ensure-configuration:
	touch infra.yml && \
	touch secrets.yml

# JuiceFS

format-dfs:
	docker run --network kind onnovalkering/juicefs \
		format \
		--access-key minio \
		--secret-key minio123 \
		--storage minio \
		--bucket http://minio:9000/data \
		redis \
		brane

# TODO: move below to contrib / seperate repository

# Kubernetes in Docker (kind)

install-kind:
	./contrib/kind/install-kubectl.sh && \
	./contrib/kind/install-kind.sh

create-kind-network:
	if [ ! -n "$(shell docker network ls -f name=kind | grep kind)" ]; then \
		docker network create kind; \
	fi;

create-kind-cluster:
	kind create cluster --config=contrib/kind/config.yml --wait 5m

delete-kind-cluster:
	kind delete cluster --name brane

kind-cluster-config:
	@kind get kubeconfig --internal --name brane | base64

# Slurm

start-slurm: create-kind-network
	docker run --rm -dt \
		--privileged \
		--network kind \
		--name slurm \
		-p 127.0.0.1:10022:22 \
		onnovalkering/slurm

# JupyterLab IDE

jupyterlab-token:
	@docker logs brane_brane-ide_1 2>&1 \
	| grep "token=" \
	| tail -1 \
	| sed "s#.*token=##"

