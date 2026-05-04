use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

static RELEASE_ROOT: &str = "https://github.com/pykeio/kv8/releases/download/";

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

    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let lib_name = if target_os != "windows" {
        "libkv8"
    } else {
        "kv8"
    };
    let lib_suffix = if target_os != "windows" { ".a" } else { ".lib" };

    let binding_url = format!("{RELEASE_ROOT}/v{version}/binding_{target}.rs");
    let binding_path = out_dir.join("binding.rs");
    if !binding_path.exists() {
        let Some(true) = try_download(&binding_url, &binding_path) else {
            panic!("downloading {binding_url} failed; target {target} may be unsupported");
        };
    }

    let lib_url = format!("{RELEASE_ROOT}/v{version}/{lib_name}_{target}{lib_suffix}.gz");
    let lib_path = out_dir.join(format!("{lib_name}{lib_suffix}"));
    if !lib_path.exists() {
        let Some(true) = try_download(&lib_url, &lib_path) else {
            panic!("downloading {lib_url} failed");
        };
    }

    println!(
        "cargo:rustc-env=RUSTY_V8_SRC_BINDING_PATH={}",
        binding_path.display()
    );
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=kv8");
}

fn try_download(url: &str, filename: &Path) -> Option<bool> {
    for (method, downloader) in [
        ("deno", try_download_deno as fn(&str, &Path) -> Option<bool>),
        ("node", try_download_node),
        ("python", try_download_python),
    ] {
        println!("attempting to download with {method}");
        if let Some(ret) = downloader(url, filename) {
            return Some(ret);
        }
    }
    None
}

const DENO_SCRIPT: &str = "let[u,p]=Deno.args;let r=await fetch(u);if(!r.ok){console.error(r.status);Deno.exit(1)};let o=await Deno.open(p,{write:true,create:true});await (u.endsWith('.gz')?r.body.pipeThrough(new DecompressionStream('gzip')):r.body).pipeTo(o.writable)";
fn try_download_deno(url: &str, out: &Path) -> Option<bool> {
    Command::new("deno")
        .arg("eval")
        .arg(DENO_SCRIPT)
        .arg("--")
        .arg(url)
        .arg(&out)
        .status()
        .ok()
        .map(|s| s.success())
}

const NODE_SCRIPT: &str = "var H=require('https'),Z=require('zlib'),F=require('fs'),A=process.argv.slice(1);function e(e){console.error(e);process.exit(1)}function h(r){if(r.statusCode!=200)e(r.statusCode);var f=F.openSync(A[1],'w'),d=Z.createGunzip();(A[0].slice(-3)=='.gz'?r.pipe(Z.createGunzip()):r).on('data',function(d){F.writeSync(f,d)}).on('close',function(){F.closeSync(f)}).on('error',e)}H.get(A[0],function(r){if(r.statusCode==302)H.get(r.headers.location,h);else h(r)}).on('error',e)";
fn try_download_node(url: &str, out: &Path) -> Option<bool> {
    Command::new("node")
        .arg("-e")
        .arg(NODE_SCRIPT)
        .arg(url)
        .arg(&out)
        .status()
        .ok()
        .map(|s| s.success())
}

const PYTHON_SCRIPT: &str = r#"from sys import argv;from zlib import decompressobj,MAX_WBITS
try:from urllib2 import HTTPError,URLError,urlopen
except ImportError:from urllib.error import HTTPError,URLError;from urllib.request import urlopen
r=urlopen(argv[1]);d=decompressobj(16+MAX_WBITS)
with open(argv[2],'wb')as o:
	while True:
		c=r.read(4096)
		if not c:break
		o.write(d.decompress(c)if argv[2].endswith('.gz')else c)
	o.write(d.flush())"#;
fn try_download_python(url: &str, out: &Path) -> Option<bool> {
    for py in ["python", "python3", "python2"] {
        if let Some(res) = Command::new(py)
            .arg("-c")
            .arg(PYTHON_SCRIPT)
            .arg(url)
            .arg(&out)
            .status()
            .ok()
            .map(|s| s.success())
        {
            return Some(res);
        }
    }
    None
}
