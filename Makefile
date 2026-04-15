SHELL := /bin/bash

MLX_VERSION ?= v0.31.1
MLX_REPO ?= https://github.com/ml-explore/mlx.git
MLX_SRC_DIR ?= $(CURDIR)/.deps/mlx
MLX_BUILD_DIR ?= $(MLX_SRC_DIR)/build
MLX_PREFIX ?= $(CURDIR)/.local/apple-mlx

export CMAKE_PREFIX_PATH := $(MLX_PREFIX)
export MLX_DIR := $(MLX_PREFIX)/share/cmake/MLX

.PHONY: help install-tools install-metal check-metal clone-mlx build-mlx install-mlx \
	build test run run-example run-complex examples-check clean clean-mlx print-env

help:
	@echo "Targets:"
	@echo "  make install-tools    # install/verify local prerequisites"
	@echo "  make install-metal    # install Apple Metal toolchain"
	@echo "  make check-metal      # verify Metal toolchain"
	@echo "  make clone-mlx        # clone upstream MLX into .deps/mlx"
	@echo "  make build-mlx        # configure and build upstream MLX"
	@echo "  make install-mlx      # install upstream MLX into .local/apple-mlx"
	@echo "  make build            # cargo build using installed MLX"
	@echo "  make test             # cargo test using installed MLX"
	@echo "  make run              # cargo run using installed MLX"
	@echo "  make run-complex      # cargo run --example complex_matmul"
	@echo "  make run-example EXAMPLE=example_graph"
	@echo "  make examples-check   # cargo check --examples"
	@echo "  make clean            # cargo clean"
	@echo "  make clean-mlx        # remove local MLX clone/build/install"
	@echo "  make print-env        # print MLX-related environment"

install-tools:
	xcode-select -p >/dev/null
	command -v cmake >/dev/null
	command -v cargo >/dev/null

install-metal:
	./scripts/install-metal-toolchain.sh

check-metal:
	./scripts/check-metal-toolchain.sh

clone-mlx:
	mkdir -p "$(dir $(MLX_SRC_DIR))"
	if [ ! -d "$(MLX_SRC_DIR)/.git" ]; then \
		git clone --depth 1 --branch "$(MLX_VERSION)" "$(MLX_REPO)" "$(MLX_SRC_DIR)"; \
	else \
		git -C "$(MLX_SRC_DIR)" fetch --depth 1 origin "$(MLX_VERSION)"; \
		git -C "$(MLX_SRC_DIR)" checkout "$(MLX_VERSION)"; \
	fi

build-mlx: clone-mlx
	cmake -S "$(MLX_SRC_DIR)" -B "$(MLX_BUILD_DIR)" \
		-DCMAKE_BUILD_TYPE=Release \
		-DMLX_BUILD_TESTS=OFF \
		-DMLX_BUILD_EXAMPLES=OFF \
		-DMLX_BUILD_BENCHMARKS=OFF \
		-DMLX_BUILD_PYTHON_BINDINGS=OFF \
		-DMLX_BUILD_METAL=$(shell if xcrun -sdk macosx metal -v >/dev/null 2>&1; then echo ON; else echo OFF; fi)
	cmake --build "$(MLX_BUILD_DIR)" -j

install-mlx: build-mlx
	cmake --install "$(MLX_BUILD_DIR)" --prefix "$(MLX_PREFIX)"

build:
	test -d "$(MLX_DIR)"
	cargo build

test:
	test -d "$(MLX_DIR)"
	cargo test

run:
	test -d "$(MLX_DIR)"
	cargo run

run-complex:
	test -d "$(MLX_DIR)"
	cargo run --example complex_matmul

run-example:
	test -n "$(EXAMPLE)"
	test -d "$(MLX_DIR)"
	cargo run --example "$(EXAMPLE)"

examples-check:
	test -d "$(MLX_DIR)"
	cargo check --examples

clean:
	cargo clean

clean-mlx:
	rm -rf "$(CURDIR)/.deps" "$(CURDIR)/.local"

print-env:
	@echo "MLX_VERSION=$(MLX_VERSION)"
	@echo "MLX_SRC_DIR=$(MLX_SRC_DIR)"
	@echo "MLX_BUILD_DIR=$(MLX_BUILD_DIR)"
	@echo "MLX_PREFIX=$(MLX_PREFIX)"
	@echo "CMAKE_PREFIX_PATH=$(CMAKE_PREFIX_PATH)"
	@echo "MLX_DIR=$(MLX_DIR)"
