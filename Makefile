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
start-instance: \
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

start-brane:
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-brn.yml up -d

stop-brane:
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-brn.yml down

start-ide:
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-ide.yml up -d

stop-ide:
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-ide.yml down

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

# JupyterLab IDE

jupyterlab-token:
	@docker logs brane_brane-ide_1 2>&1 \
	| grep "token=" \
	| tail -1 \
	| sed "s#.*token=##"
