use std::env;

fn main() {
    for var in ["CXXSTDLIB", "KV8_BINDING_PATH", "KV8_BUILD_DIR"] {
        println!("cargo:rerun-if-env-changed={var}");
    }

    println!(
        "cargo:rustc-check-cfg=cfg(feature, values(\"v8_enable_sandbox\", \"v8_enable_v8_checks\"))"
    );
    for feature in ["simdutf", "v8_enable_pointer_compression"] {
        println!("cargo:rustc-check-cfg=cfg(feature, values(\"{feature}\"))");
        println!("cargo:rustc-cfg=feature=\"{feature}\"");
    }

    if let Ok(stdlib) = env::var("CXXSTDLIB")
        && !stdlib.is_empty()
    {
        println!("cargo:rustc-link-lib=dylib={stdlib}");
    } else {
        let target = env::var("TARGET").unwrap();
        if target.contains("apple") || target.contains("freebsd") || target.contains("openbsd") {
            println!("cargo:rustc-link-lib=dylib=c++");
        } else if target.contains("android") {
            println!("cargo:rustc-link-lib=dylib=c++_shared");
        } else if !target.contains("msvc") {
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "windows" {
        println!("cargo:rustc-link-lib=dylib=winmm");
        println!("cargo:rustc-link-lib=dylib=dbghelp");
        println!("cargo:rustc-link-lib=dylib=msvcprt");
    }

    if let Ok(binding) = env::var("KV8_BINDING_PATH")
        && let Ok(lib) = env::var("KV8_BUILD_DIR")
    {
        println!("cargo:rustc-env=RUSTY_V8_SRC_BINDING_PATH={binding}");
        println!("cargo:rustc-link-search={lib}");
        println!("cargo:rustc-link-lib=kv8");
        return;
    }
}
