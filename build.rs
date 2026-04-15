use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const VENDORED_MLX_C_DIR: &str = "vendor/mlx-c";

fn main() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let source_dir = manifest_dir.join(VENDORED_MLX_C_DIR);
    let docs_only =
        env::var_os("CARGO_FEATURE_DOCS_ONLY").is_some() || env::var_os("DOCS_RS").is_some();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", source_dir.display());
    println!("cargo:rerun-if-env-changed=CMAKE_PREFIX_PATH");
    println!("cargo:rerun-if-env-changed=MLX_DIR");
    println!("cargo:rerun-if-env-changed=MLX_BUILD_METAL");
    println!("cargo:rerun-if-env-changed=DOCS_RS");

    generate_bindings(&source_dir, &out_dir);

    if docs_only {
        return;
    }

    let metal_enabled = has_metal_toolchain();

    if !metal_enabled {
        println!("cargo:warning=Metal toolchain not available; building MLX with CPU backend only");
    }

    let mut cfg = cmake::Config::new(&source_dir);
    cfg.profile("Release")
        .define("MLX_C_BUILD_EXAMPLES", "OFF")
        .define("MLX_C_USE_SYSTEM_MLX", "ON")
        .define("BUILD_SHARED_LIBS", "ON")
        .define("MLX_BUILD_METAL", if metal_enabled { "ON" } else { "OFF" });

    let mut system_lib_dirs = Vec::new();

    if let Some(prefix) = env::var_os("CMAKE_PREFIX_PATH") {
        let prefix = prefix.to_string_lossy().into_owned();
        cfg.define("CMAKE_PREFIX_PATH", &prefix);
        system_lib_dirs.extend(prefix_lib_dirs(&prefix));
    }

    if let Some(mlx_dir) = env::var_os("MLX_DIR") {
        let mlx_dir = PathBuf::from(&mlx_dir);
        cfg.define("MLX_DIR", mlx_dir.to_string_lossy().as_ref());
        if let Some(prefix) = mlx_prefix_from_mlx_dir(&mlx_dir) {
            system_lib_dirs.push(prefix.join("lib"));
        }
    }

    let dst = cfg.build();
    let lib_dir = dst.join("lib");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    for dir in system_lib_dirs {
        if dir.exists() {
            println!("cargo:rustc-link-search=native={}", dir.display());
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dir.display());
        }
    }
    println!("cargo:rustc-link-lib=dylib=mlxc");
    println!("cargo:rustc-link-lib=dylib=mlx");
    println!("cargo:rustc-link-lib=dylib=c++");
    println!("cargo:rustc-link-lib=framework=Accelerate");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());

    if metal_enabled {
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=QuartzCore");
    }
}

fn generate_bindings(source_dir: &Path, out_dir: &Path) {
    let header = source_dir.join("mlx/c/mlx.h");
    let include_dir = source_dir.to_string_lossy().into_owned();

    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .clang_arg(format!("-I{include_dir}"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_file(".*/mlx/c/.*")
        .layout_tests(false)
        .generate_comments(false)
        .generate()
        .expect("failed to generate mlx-c bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("failed to write bindings.rs");
}

fn has_metal_toolchain() -> bool {
    if let Some(value) = env::var_os("MLX_BUILD_METAL") {
        let value = value.to_string_lossy();
        if value.eq_ignore_ascii_case("on") || value == "1" || value.eq_ignore_ascii_case("true") {
            return true;
        }
        if value.eq_ignore_ascii_case("off") || value == "0" || value.eq_ignore_ascii_case("false")
        {
            return false;
        }
    }

    Command::new("xcrun")
        .args(["-sdk", "macosx", "metal", "-v"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn prefix_lib_dirs(prefixes: &str) -> Vec<PathBuf> {
    prefixes
        .split([';', ':'])
        .filter(|entry| !entry.is_empty())
        .map(PathBuf::from)
        .map(|prefix| prefix.join("lib"))
        .collect()
}

fn mlx_prefix_from_mlx_dir(mlx_dir: &Path) -> Option<PathBuf> {
    let mut path = mlx_dir.to_path_buf();
    for _ in 0..3 {
        path = path.parent()?.to_path_buf();
    }
    Some(path)
}
