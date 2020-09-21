.PHONY: all

all: api cli init loop jupyter

api:
	docker build -t onnovalkering/brane-api -f Dockerfile.api .

cli:
	cargo build --release --package brane-cli

init:
	cargo build --release --package brane-init --target x86_64-unknown-linux-musl

jupyter:
	docker build -t onnovalkering/brane-jupyterlab brane-ide/jupyterlab

loop:
	docker build -t onnovalkering/brane-loop -f Dockerfile.loop .

start:
	docker-compose -f deployment/docker/docker-compose.yml up -d

stop:
	docker-compose -f deployment/docker/docker-compose.yml down
