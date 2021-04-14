build: binaries docker


# Build release binaries
binaries: cli init

cli:
	cargo build --release --package brane-cli

init:
	cargo build --release --package brane-init --target x86_64-unknown-linux-musl

branelet:
	cargo build --release --package brane-let --target x86_64-unknown-linux-musl


# Build Docker images
docker: api ide loop

api:
	docker build -t onnovalkering/brane-api -f Dockerfile.api .

clb:
	docker build -t onnovalkering/brane-clb -f Dockerfile.clb .

ide:
	docker build -t onnovalkering/brane-ide -f Dockerfile.ide .

loop:
	docker build -t onnovalkering/brane-loop -f Dockerfile.loop .

noop:
	docker build -t onnovalkering/brane-noop -f Dockerfile.noop .


# Development setup
start: start-services
	tmux new-session -d -s brane 				&& \
 	tmux rename-window 'Brane'					&& \
	tmux send-keys 'make start-loop' 'C-m'		&& \
	tmux split-window -h						&& \
	tmux send-keys 'make start-api' 'C-m'		&& \
	tmux split-window -v						&& \
	tmux send-keys 'make start-ide' 'C-m'		&& \
	tmux -2 attach-session -t brane

stop: stop-services
	tmux select-window -t brane:0				&& \
	tmux select-pane -t 0 						&& \
	tmux send-keys 'C-c'						&& \
	tmux select-pane -t 1 						&& \
	tmux send-keys 'C-c'						&& \
	tmux select-pane -t 2 						&& \
	tmux send-keys 'C-c' 'y' 'C-m'				&& \
	sleep 1										&& \
	tmux kill-session -t brane

start-api:
	cd brane-api && cargo run

start-ide:
	cd brane-ide && make start

start-loop:
	cd brane-loop && cargo run

start-services: create-kind-network
	docker-compose up -d

stop-services:
	docker-compose down

restart-services: stop-services start-services

# Kubernetes in Docker (kind)

install-kind:
	./contrib/kind/install-kubectl.sh && \
	./contrib/kind/install-kind.sh

create-kind-network:
	@if [ ! -n "$(shell docker network ls -f name=kind | grep kind)" ]; then \
		docker network create kind; \
	fi;

create-kind-cluster:
	kind create cluster --config=contrib/kind/config.yml --wait 5m

delete-kind-cluster:
	kind delete cluster --name brane

kind-cluster-config:
	@kind get kubeconfig --name brane | base64
