include .env

all: build

release:
	docker build -t github-deployment .
	docker run -v "$(PWD)/target/release:/tmp/release" \
	           -e "GITHUB_API_TOKEN=$(GITHUB_API_TOKEN)" \
	           github-deployment make

build:
	cargo build

run: build
	./target/debug/github-deployment
