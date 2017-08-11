include .env

TARGET ?= x86_64-unknown-linux-gnu

revision = $(shell git log -1 --pretty=format:%h)
version = $(shell grep -E "const VERSION" src/main.rs | grep -oE --color=never "\d\.\d")

all: build

release:
	docker build --force-rm \
	             -f docker/$(TARGET)/Dockerfile \
	             -t github-deployment .
	docker run -v "$(PWD)/releases:/tmp/release" \
	           -v "$(PWD)/src:/tmp/src" \
	           -v "$(PWD)/Cargo.toml:/tmp/Cargo.toml" \
	           -e "TARGET=$(TARGET)" \
	           -e "REVISION=$(revision)" \
	           -e "RELEASE_BIN_NAME=github-deployment-$(version)" \
	           -e "GITHUB_API_TOKEN=$(GITHUB_API_TOKEN)" \
	           github-deployment make

build:
	REVISION=$(revision) cargo build
