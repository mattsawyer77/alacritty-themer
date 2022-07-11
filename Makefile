SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

binary-name                 := alacritty-themer
RUST_VERSION                ?= 1.62.0
RUST_BUILD_IMAGE            ?= fredrikfornwall/rust-static-builder
debug-binary                := target/debug/${binary-name}
linux-release-binary        := target/release${binary-name}
version                     := $(shell grep '^version' Cargo.toml | cut -d'"' -f2)
static-linux-release-binary := target/x86_64-unknown-linux-musl/release/${binary-name}
mac-release-binary          := target/release/${binary-name}
static-build-tagged-image   := ${RUST_BUILD_IMAGE}:${RUST_VERSION}
sources                      = $(shell find src -name '*.rs') Cargo.toml Cargo.lock
linux-checksum               = $(shell md5sum "${static-linux-release-binary}" | awk '{ print $$1 }')
mac-checksum                 = $(shell md5sum "${mac-release-binary}" | awk '{ print $$1 }')

setup:
	command -v cargo >/dev/null || (curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh)
	rustup override set ${RUST_VERSION}
.PHONY: setup

reset-themes:
	touch themes/*

debug: ${debug-binary}
.PHONY: debug

linux-release: ${linux-release-binary}
.PHONY: linux-release

mac-release: ${mac-release-binary}
.PHONY: mac-release

mac-install: ${mac-release-binary}
	cargo install --path .
.PHONY: mac-install

install: ${linux-release-binary}
	cargo install --path .
.PHONY: install

static: ${static-linux-release-binary}
.PHONY: static

clean: target
	rm -rf target

${debug-binary}: ${sources}
	cargo build

${linux-release-binary}: ${sources}
	cargo build --release

${mac-release-binary}: ${sources}
	cargo build --release

${static-linux-release-binary}: ${sources}
	docker run \
		-v "$$(pwd)":/build \
		-v $$HOME/.cargo/git:/root/.cargo/git \
		-v $$HOME/.cargo/registry:/root/.cargo/registry \
		${static-build-tagged-image}
