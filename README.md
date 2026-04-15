# apple-mlx

Rust bindings for Apple MLX through the official `mlx-c` C API.

This crate currently provides:

- `apple_mlx::raw`: full generated raw bindings for `mlx-c`
- a thin safe layer for `Device`, `Stream`, `Array`, and `Complex32`
- a working complex matrix multiplication example validated against a CPU reference

## Status

This crate is now library-usable and packageable for crates.io.

The important build constraint is explicit:

- `mlx-c` is vendored in this crate
- MLX itself is not fetched at build time
- you must provide an installed MLX package and point CMake at it with `CMAKE_PREFIX_PATH` or `MLX_DIR`

That keeps the crate build reproducible and avoids hidden network fetches during `cargo build`.

## Project Layout

- `src/lib.rs`: generated raw bindings export plus thin safe wrappers
- `src/main.rs`: small binary using the library
- `examples/complex_matmul.rs`: example entrypoint for the same demo
- `build.rs`: generates bindings and builds vendored `mlx-c` against an installed MLX
- `vendor/mlx-c`: vendored upstream `mlx-c` source

## Library Surface

Raw bindings:

```rust
use apple_mlx::raw;
```

Thin safe API:

```rust
use apple_mlx::{Array, Complex32, Device, Stream};
```

Demo entrypoint:

```rust
apple_mlx::demo_complex_matmul()?;
```

## How the Build Works

`build.rs` does three things:

1. Generates Rust bindings from `vendor/mlx-c/mlx/c/mlx.h` using `bindgen`.
2. Builds vendored `mlx-c` with CMake.
3. Links it against an already-installed MLX package discovered through:
   - `CMAKE_PREFIX_PATH`
   - or `MLX_DIR`

Metal support is enabled only if this succeeds:

```bash
xcrun -sdk macosx metal -v
```

If that command fails, the build falls back to CPU-only MLX usage.

## Requirements

- macOS on Apple silicon
- Rust toolchain
- Xcode command line tools
- CMake
- an installed MLX package

Install the basic tools if needed:

```bash
xcode-select --install
brew install cmake
rustup toolchain install stable
```

## Recommended Setup

The easiest path is to let the repo `Makefile` install MLX into a local prefix and wire the environment for you.

Install upstream MLX into `.local/apple-mlx`:

```bash
make install-mlx
```

Then build and run this crate:

```bash
make build
make test
make run
```

Run a specific example:

```bash
make run-example EXAMPLE=example_graph
```

## Metal Toolchain Setup

This repo includes helper scripts for the Metal compiler toolchain:

Install or repair the toolchain:

```bash
./scripts/install-metal-toolchain.sh
```

Verify the toolchain:

```bash
./scripts/check-metal-toolchain.sh
```

The expected verification command is:

```bash
xcrun -sdk macosx metal -v
```

If that command fails, `build.rs` will print:

```text
Metal toolchain not available; building MLX with CPU backend only
```

If it succeeds, the crate can build the GPU-capable path.

## Installing MLX for This Crate

This crate expects MLX to be installed somewhere CMake can find it.

The default `Makefile` flow installs MLX here:

```bash
$(pwd)/.local/apple-mlx
```

and exports:

```bash
CMAKE_PREFIX_PATH="$(pwd)/.local/apple-mlx"
MLX_DIR="$(pwd)/.local/apple-mlx/share/cmake/MLX"
```

If you do not use the `Makefile`, you must provide those paths yourself.

## Build and Run

All commands below assume you are in the crate root:

```bash
ls Cargo.toml build.rs src/lib.rs
```

Build:

```bash
make build
```

Run the binary:

```bash
make run
```

Run the example:

```bash
make run-complex
```

Run tests:

```bash
make test
```

## Verified CPU Run

This environment was verified successfully in CPU mode. The Metal toolchain was not installed, so the crate built and ran with CPU fallback.

Observed output:

```text
Using Apple MLX on CPU device 0 (Apple M2 Pro)
Output shape: [2, 2]
Left matrix:
  1.000+2.000i  3.000-1.000i
  -2.000+0.500i  0.000+4.000i
Right matrix:
  0.500-1.000i  2.000+0.000i
  -3.000+1.500i  1.000-2.000i
MLX product:
  -5.000+7.500i  3.000-3.000i
  -6.500-9.750i  4.000+5.000i
Max absolute error vs CPU reference: 0.000000
```

## GPU Reproduction

To run on GPU, install the Metal toolchain first:

```bash
./scripts/install-metal-toolchain.sh
./scripts/check-metal-toolchain.sh
```

Then rebuild and run with the same MLX prefix:

```bash
make install-metal
make check-metal
make clean
make build
make run
```

If GPU support is available, the program should print:

```text
Using Apple MLX on GPU device 0 (...)
```

## Packaging Notes

- package name: `apple-mlx`
- crate docs target: `docs.rs/apple-mlx`
- docs.rs uses the `docs-only` feature to skip native library compilation during documentation builds
- no build-time network fetches are required by this crate

## Current Limits

- the safe wrapper only covers a small subset of MLX so far
- the raw `mlx-c` binding surface is available, but ergonomic Rust wrappers still need to be expanded module by module
- GPU execution was designed for and wired in, but only CPU execution was verified in this environment
