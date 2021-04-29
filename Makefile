build: build-binaries build-docker

# Build release binaries
build-binaries: build-cli build-let

build-cli:
	cargo build --release --package brane-cli

build-let:
	cargo build --release --package brane-let --target x86_64-unknown-linux-musl

# Build Docker images
build-images: api clb drv ide job log plr

build-api-image:
	docker build -t onnovalkering/brane-api -f Dockerfile.api .

build-clb-image:
	docker build -t onnovalkering/brane-clb -f Dockerfile.clb .

build-drv-image:
	docker build -t onnovalkering/brane-drv -f Dockerfile.drv .

build-ide-image:
	docker build -t onnovalkering/brane-ide -f Dockerfile.ide .

build-job-image:
	docker build -t onnovalkering/brane-job -f Dockerfile.job .

build-log-image:
	docker build -t onnovalkering/brane-log -f Dockerfile.log .

build-plr-image:
	docker build -t onnovalkering/brane-plr -f Dockerfile.plr .

# Development setup
start: \
	create-kind-network \
	start-support \
	format-dfs \
	start-brane

start-support:
	docker-compose -f docker-compose-support.yml up -d

start-brane:
	docker-compose -f docker-compose-brane.yml up -d

stop: \
	stop-support \
	stop-brane

stop-support:
	docker-compose -f docker-compose-support.yml down

stop-brane:
	docker-compose -f docker-compose-brane.yml down

restart: stop start

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

# JuiceFS

format-dfs:
	@docker run --network kind onnovalkering/juicefs \
		format \
		--access-key minio \
		--secret-key minio123 \
		--storage minio \
		--bucket http://minio:9000/data \
		redis \
		brane

# JupyterLab

jupyterlab-token:
	@docker logs brane_brane-ide_1 2>&1 \
	| grep "token=" \
	| tail -1 \
	| sed "s#.*token=##"
