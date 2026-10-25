use std::{env, fs, path::PathBuf};

fn main() {
  let Ok(list) = env::var("DEP_KV8_SYMBOLS") else {
    return;
  };

  println!("cargo::rerun-if-changed={list}");

  let symbols = fs::read_to_string(&list).unwrap();
  let symbols = symbols.lines().filter(|s| !s.is_empty());
  let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

  if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
    for symbol in symbols {
      println!("cargo::rustc-link-arg=/EXPORT:{symbol}");
    }
  } else if env::var("CARGO_CFG_TARGET_VENDOR").as_deref() == Ok("apple") {
    let file = out_dir.join("exports.txt");
    fs::write(
      &file,
      symbols
        .map(|s| format!("_{s}"))
        .collect::<Vec<_>>()
        .join("\n"),
    )
    .unwrap();
    println!(
      "cargo::rustc-link-arg=-Wl,-exported_symbols_list,{}",
      file.display()
    );
  } else {
    // works with rust-lld, dont know about gnu ld but hopefully there are no psychos using it anyway
    let file = out_dir.join("exports.map");
    fs::write(
      &file,
      format!(
        "{{\n global:\n{}\n}};\n",
        symbols
          .map(|s| format!("  {s};"))
          .collect::<Vec<_>>()
          .join("\n")
      ),
    )
    .unwrap();
    println!(
      "cargo::rustc-link-arg=-Wl,--version-script={}",
      file.display()
    );
  }
}
