.PHONY: all

all: brane-api brane-cli brane-init brane-loop brane-jupyter

brane-api:
	docker build -t onnovalkering/brane-api brane-api

brane-cli:
	cargo build --release --package brane-cli

brane-init:
	cargo build --release --package brane-init --target x86_64-unknown-linux-musl

brane-jupyter:
	docker build -t onnovalkering/brane-jupyterlab brane-ide/jupyterlab

brane-loop:
	docker build -t onnovalkering/brane-loop brane-loop

