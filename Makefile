.PHONY: all

all: api cli ide init loop

api:
	docker build -t onnovalkering/brane-api -f Dockerfile.api .

cli:
	cargo build --release --package brane-cli

ide:
	docker build -t onnovalkering/brane-ide -f Dockerfile.ide .

init:
	cargo build --release --package brane-init --target x86_64-unknown-linux-musl

loop:
	docker build -t onnovalkering/brane-loop -f Dockerfile.loop .

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

restart-services: stop-services start-services
