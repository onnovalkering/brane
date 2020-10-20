.PHONY: all

# Build release binaries
binaries: cli init

cli:
	cargo build --release --package brane-cli

init:
	cargo build --release --package brane-init --target x86_64-unknown-linux-musl

# Build Docker images
docker: api ide loop

api:
	docker build -t onnovalkering/brane-api -f Dockerfile.api .

ide:
	docker build -t onnovalkering/brane-ide -f Dockerfile.ide .

loop:
	docker build -t onnovalkering/brane-loop -f Dockerfile.loop .

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

stop: 
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
	cd brane-ide && API_HOST="`hostname`:8080" pipenv shell "jupyter lab --ip 0.0.0.0 --LabApp.token=''"

start-loop:
	cd brane-loop && cargo run

start-services:
	docker-compose up -d

stop-services:
	docker-compose down
