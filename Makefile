build: build-binaries build-services

##############
## BINARIES ##
##############

build-binaries: \
	build-cli \
	build-let

build-cli:
	cargo build --release --package brane-cli

build-let:
	rustup target add x86_64-unknown-linux-musl
	cargo build --release --package brane-let --target x86_64-unknown-linux-musl

##############
## SERVICES ##
##############

build-services: \
	build-api-image \
	build-clb-image \
	build-drv-image \
	build-job-image \
	build-log-image \
	build-plr-image

build-api-image:
	docker build --load -t ghcr.io/onnovalkering/brane/brane-api -f Dockerfile.api .

build-clb-image:
	docker build --load -t ghcr.io/onnovalkering/brane/brane-clb -f Dockerfile.clb .

build-drv-image:
	docker build --load -t ghcr.io/onnovalkering/brane/brane-drv -f Dockerfile.drv .

build-job-image:
	docker build --load -t ghcr.io/onnovalkering/brane/brane-job -f Dockerfile.job .

build-log-image:
	docker build --load -t ghcr.io/onnovalkering/brane/brane-log -f Dockerfile.log .

build-plr-image:
	docker build --load -t ghcr.io/onnovalkering/brane/brane-plr -f Dockerfile.plr .

##############
## INSTANCE ##
##############

start-instance: \
	ensure-docker-images \
	ensure-docker-network \
	ensure-configuration \
	start-svc \
	start-brn

stop-instance: \
	stop-brn \
	stop-svc

ensure-docker-images:
	if [ -z "${BRANE_VERSION}" ]; then \
		make build-services; \
	fi;

ensure-docker-network:
	if [ ! -n "$(shell docker network ls -f name=brane | grep brane)" ]; then \
		docker network create brane; \
	fi;

ensure-configuration:
	touch infra.yml && \
	touch secrets.yml	

start-svc:
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-svc.yml up -d
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-svc.yml rm -f

stop-svc:
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-svc.yml down

start-brn:
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-brn.yml up -d

stop-brn:
	COMPOSE_IGNORE_ORPHANS=1 docker-compose -p brane -f docker-compose-brn.yml down
