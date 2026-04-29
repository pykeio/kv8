fn main() {
    println!(
        "cargo:rustc-check-cfg=cfg(feature, values(\"v8_enable_sandbox\", \"v8_enable_v8_checks\"))"
    );
    for feature in ["simdutf", "v8_enable_pointer_compression"] {
        println!("cargo:rustc-check-cfg=cfg(feature, values(\"{feature}\"))");
        println!("cargo:rustc-cfg=feature=\"{feature}\"");
    }
}
