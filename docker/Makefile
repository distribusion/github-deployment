TARGET ?= x86_64-unknown-linux-gnu
ROOT_PATH ?= .
TARGET_BIN_NAME ?= unknown
RELEASE_BIN_NAME ?= unknown

target-path = $(ROOT_PATH)/target/$(TARGET)
target-bin-path = $(target-path)/release/$(BIN_NAME)
release-path = $(ROOT_PATH)/release/$(TARGET)
release-bin-path = $(release-path)/$(RELEASE_BIN_NAME)

all: $(release-bin-path)

$(release-bin-path): $(target-bin-path) | $(release-path)
	cp $^ $@

$(target-bin-path): src/main.rs Cargo.toml | $(target-path)
	cargo build --release --target $(TARGET)

$(release-path):
	mkdir -p $@

$(target-path):
	mkdir -p $@
