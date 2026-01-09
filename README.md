# Raw FFI Interface for Saucer

This repository provides raw Rust bindings for [saucer](https://github.com/saucer/saucer)
via [bindgen](https://rust-lang.github.io/rust-bindgen/). It provides corresponding FFI interfaces and handles most of,
if not all, the details required to build and link saucer.

It's possible to build an app directly using these raw bindings, but you would normally want to
use [saucers](https://github.com/skjsjhb/saucers), which provides a safe wrapper.

## Cargo Features

This crate comes with reasonable defaults, which should be sufficient for most usages.
Several features can be used to customize the build:

- `gen-bindings`: Do not use the vendored bindings (shipped with the sources), generate one on the fly.

- `qt`: Use the Qt6 backend instead of the platform default.

- `lto`: Enable LTO for static linking. Additional steps are required to make LTO actually work, see instructions below.

- `shared-lib`: Build a shared library.

> [!WARNING]
>
> The produced shared libraries are put inside the build directory of this crate, which is (normally) inaccessible
> from binary crates that depends on it. This crate points the env `SAUCERS_OUT_DIR` to the output directory, thus one
> may copy the shared libraries as needed. However, it's generally not a good idea to enable dynamic linking for this
> crate.

## Build Performance

On Windows, CMake will pick up Visual Studio (MSBuild) by default, which takes more than minutes to compile the C++
code. You can speed up this by explicitly setting the generator via an env variable in `.cargo/config.toml`:

```toml
[env]
SAUCERS_CMAKE_GENERATOR_x86_64-pc-windows-msvc = "Ninja"
```

## Link Time Optimization

Rust (mainly the Cargo toolchain) is designed to work under static linking for third-party libraries. Static linking,
however, would increase the binary size slightly. LTO can be used to reduce such overhead:

1. Install `clang`. Make sure `rustc` and `clang` shares the same major LLVM version (or you'll get link errors):

   ```shell
   rustc -vV    # LLVM version: 21.1.3
   clang -v     # clang version 21.1.8
   ```

   `clang-cl` is also needed if building on Windows.

2. Instruct CMake to use `clang` as the compiler. This crate provides several env variables that can be used for this
   purpose, configurable in `.cargo/config.toml`:

    ```toml
    [env]
    SAUCERS_CMAKE_CXX_COMPILER = "clang"
    SAUCERS_CMAKE_AR = "llvm-ar"
    ```

   Alternatively, use target suffix for a specific target:

    ```toml 
    [env]
    SAUCERS_CMAKE_CXX_COMPILER_x86_64-pc-windows-msvc = "clang-cl"
    SAUCERS_CMAKE_AR_x86_64-pc-windows-msvc = "llvm-lib"
    ```

3. Use `lld` (`lld-link` on Windows) to link the Rust part,
   see [Linker-plugin-based LTO](https://doc.rust-lang.org/beta/rustc/linker-plugin-lto.html) for details.

   ```toml
    [target.'cfg(target_os = "windows")']
    linker = "lld-link"
    rustflags = ["-Clinker-plugin-lto"]

    [target.'cfg(any(target_os = "linux", target_os = "macos"))']
    linker = "clang"
    rustflags = ["-Clinker-plugin-lto", "-Clink-arg=-fuse-ld=lld"]
   ```
