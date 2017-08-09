TARGET ?= x86_64-unknown-linux-gnu
BIN_NAME ?= unknown
ROOT_PATH ?= .
target-path = $(ROOT_PATH)/target/$(TARGET)
target-bin-path = $(target-path)/release/$(BIN_NAME)
release-path = $(ROOT_PATH)/release
release-bin-path = $(release-path)/$(BIN_NAME)

all: $(release-bin-path)

$(release-bin-path): $(target-bin-path) | $(release-path)
	cp $^ $@

$(target-bin-path): src/main.rs Cargo.toml | $(target-path)
	cargo build --release --target $(TARGET)

$(release-path):
	mkdir -p $@

$(target-path):
	mkdir -p $@
