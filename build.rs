use build_target::{Arch, Env, Os};
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("`OUT_DIR` should be set"));
    let os = build_target::target_os();

    let profile = std::env::var("PROFILE").unwrap();
    let is_debug = profile == "debug" || profile == "test";

    let gen_bindings = std::env::var("CARGO_FEATURE_GEN_BINDINGS").is_ok();
    let build_shared_lib = std::env::var("CARGO_FEATURE_SHARED_LIB").is_ok();
    let is_qt = std::env::var("CARGO_FEATURE_QT").is_ok();
    let lto = std::env::var("CARGO_FEATURE_LTO").is_ok();

    if os == Os::Windows {
        let Some(Env::Msvc) = build_target::target_env() else {
            panic!("MSVC is required for Windows builds");
        };
    }

    let mut make = cmake::Config::new("saucer-bindings");

    make.define("saucers_static", if build_shared_lib { "OFF" } else { "ON" });

    if let Some(g) = get_target_env("SAUCERS_CMAKE_GENERATOR") {
        make.generator(g);
    }

    if let Some(g) = get_target_env("SAUCERS_CMAKE_GENERATOR_TOOLSET") {
        make.generator_toolset(g);
    }

    if is_qt {
        make.define("saucer_backend", "Qt");
    }

    forward_env(&mut make, "SAUCERS_CMAKE_CXX_COMPILER", "CMAKE_CXX_COMPILER");
    forward_env(&mut make, "SAUCERS_CMAKE_AR", "CMAKE_AR");

    make.no_default_flags(true); // Let CMake handle everything

    if os == Os::Windows {
        make.cxxflag("/EHsc"); // Used by CRT
    }

    if !build_shared_lib && !is_debug && lto {
        make.define("CMAKE_INTERPROCEDURAL_OPTIMIZATION", "ON");
    }

    let cmake_out = make.build();

    println!("cargo::rustc-link-search=native={}", cmake_out.display());
    println!("cargo::rustc-env=SAUCERS_OUT_DIR={}", cmake_out.display());

    let mut static_libs = Vec::new();
    let mut shared_libs = Vec::new();
    let mut frameworks = Vec::new();

    if build_shared_lib {
        shared_libs.extend(["saucer-bindings", "saucer-bindings-desktop", "saucer-bindings-pdf"]);
    } else {
        static_libs.extend([
            "saucer",
            "saucer-bindings",
            "saucer-desktop",
            "saucer-bindings-desktop",
            "saucer-pdf",
            "saucer-bindings-pdf",
            "coco",
        ]);

        if is_qt {
            if os == Os::Windows && is_debug {
                shared_libs.extend([
                    "Qt6WebEngineWidgetsd",
                    "Qt6WebChanneld",
                    "Qt6Widgetsd",
                    "Qt6Cored",
                    "Qt6WebEngineCored",
                    "Qt6Guid",
                ]);
            } else {
                shared_libs.extend([
                    "Qt6WebEngineWidgets",
                    "Qt6WebChannel",
                    "Qt6Widgets",
                    "Qt6Core",
                    "Qt6WebEngineCore",
                    "Qt6Gui",
                ]);
            }
        }

        match os {
            Os::Windows => {
                if let Some(wv2_libs) = find_webview2_libs(&out_dir) {
                    println!("cargo::rustc-link-search=native={}", wv2_libs.display());
                }

                if !is_qt {
                    static_libs.push("WebView2LoaderStatic");
                }

                if is_debug {
                    println!("cargo::rustc-link-arg=/NODEFAULTLIB:msvcrt");
                }

                shared_libs.extend(["gdiplus", "user32", "shell32", "shlwapi", "wininet", "windowsapp"]);
            }
            Os::MacOS => {
                if !is_qt {
                    frameworks.extend(["WebKit", "CoreImage"]);
                }

                shared_libs.push("c++");
                frameworks.push("Cocoa");
            }
            Os::Linux => {
                if !is_qt {
                    pkg_config::probe_library("gtk4").unwrap();
                    pkg_config::probe_library("webkitgtk-6.0").unwrap();
                    pkg_config::probe_library("libadwaita-1").unwrap();
                    pkg_config::probe_library("json-glib-1.0").unwrap();
                }

                shared_libs.push("stdc++");
            }
            it => panic!("unsupported OS: {it}"),
        }
    }

    for lib in static_libs {
        println!("cargo::rustc-link-lib=static={lib}");
    }

    for lib in shared_libs {
        println!("cargo::rustc-link-lib=dylib={lib}");
    }

    for lib in frameworks {
        println!("cargo::rustc-link-lib=framework={lib}");
    }

    // Must run the build first to get export.h before generating
    if gen_bindings {
        let bindings = bindgen::Builder::default()
            .header("saucer.h")
            .clang_args([
                "-x",
                "c++",
                "-I./saucer-bindings/include",
                "-I./saucer-bindings/include/saucer", // Looks like the modules won't use a prefix
                "-I./saucer-bindings/modules/desktop/include",
                "-I./saucer-bindings/modules/pdf/include",
            ])
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .allowlist_item("saucer.*")
            .blocklist_item(".*native")
            .prepend_enum_name(false) // Already included
            .generate()
            .expect("failed to generate bindings");

        bindings
            .write_to_file(out_dir.join("bindings.rs"))
            .expect("failed to emit bindings");
    }
}

fn forward_env(conf: &mut cmake::Config, envs: &str, cms: &str) {
    if let Some(ev) = get_target_env(envs) {
        conf.define(cms, ev);
    }
}

fn get_target_env(name: &str) -> Option<String> {
    let suffix_name = format!("{name}_{}", build_target::target().triple);

    std::env::var(name).ok().or_else(|| std::env::var(suffix_name).ok())
}

fn find_webview2_libs(out_dir: &Path) -> Option<PathBuf> {
    let root = out_dir.join("build/_deps/saucer-build/nuget/packages");
    let pkgs = std::fs::read_dir(&root).ok()?;
    for ent in pkgs {
        if let Ok(ent) = ent
            && ent.file_name().to_string_lossy().starts_with("Microsoft.Web.WebView2")
        {
            let mut fp = ent.path().join("build/native");
            let arch = build_target::target_arch();

            let arch = match arch {
                Arch::AArch64 => "arm64",
                Arch::X86 => "x86",
                Arch::X86_64 => "x64",
                it => panic!("unsupported arch: {it}"),
            };

            fp.push(arch);
            return Some(fp);
        }
    }

    None
}
