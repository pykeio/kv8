use std::{
  collections::{BTreeSet, HashMap, HashSet},
  fs::{self, File},
  io::{Read, Seek, SeekFrom},
  path::{Path, PathBuf},
};

struct Member {
  name: String,
  data: Vec<u8>,
  next: u64,
}

impl Member {
  pub fn at(f: &mut File, pos: u64, string_table: Option<&[u8]>) -> Self {
    let mut h = [0u8; 60];
    f.seek(SeekFrom::Start(pos)).unwrap();
    f.read_exact(&mut h).unwrap();
    let raw = String::from_utf8_lossy(&h[0..16]).trim_end().to_string();
    let mut size = String::from_utf8_lossy(&h[48..58])
      .trim()
      .parse::<usize>()
      .unwrap();
    let name = if let Some(len) = raw.strip_prefix("#1/") {
      // BSD
      let len = len.parse::<usize>().unwrap();
      let mut name = vec![0u8; len];
      f.read_exact(&mut name).unwrap();
      size -= len;
      let end = name.iter().position(|&b| b == 0).unwrap_or(len);
      String::from_utf8_lossy(&name[..end]).into_owned()
    } else if let Some(idx) =
      raw.strip_prefix('/').and_then(|s| s.parse::<usize>().ok())
      && let Some(string_table) = string_table
    {
      let rest = &string_table[idx..];
      let end = rest
        .iter()
        .position(|&b| b == 0 || b == b'\n')
        .unwrap_or(rest.len());
      String::from_utf8_lossy(&rest[..end])
        .trim_end_matches('/')
        .to_string()
    } else if raw == "/" || raw == "//" {
      raw
    } else {
      raw.trim_end_matches('/').to_string()
    };
    let data_start = f.stream_position().unwrap();
    let data = if name == "/" || name == "//" || name.starts_with("__.SYMDEF") {
      let mut data = vec![0u8; size];
      f.read_exact(&mut data).unwrap();
      data
    } else {
      Vec::new()
    };
    let mut next = data_start + size as u64;
    next += next & 1;
    Member { name, data, next }
  }
}

fn coff_symbols(data: &[u8]) -> Vec<(String, u64)> {
  let n = u32::from_be_bytes(data[0..4].try_into().unwrap()) as usize;
  let names = data[n * 4 + 4..]
    .split(|&b| b == 0)
    .map(|s| String::from_utf8_lossy(s).into_owned());
  names
    .zip((0..n).map(|i| {
      u32::from_be_bytes(data[i * 4 + 4..i * 4 + 8].try_into().unwrap()) as u64
    }))
    .collect()
}

fn bsd_symbols(data: &[u8], darwin64: bool) -> Vec<(String, u64)> {
  let w = if darwin64 { 8 } else { 4 };
  let read = |at: usize| -> u64 {
    if darwin64 {
      u64::from_le_bytes(data[at..at + 8].try_into().unwrap())
    } else {
      u32::from_le_bytes(data[at..at + 4].try_into().unwrap()) as u64
    }
  };
  let size = read(0) as usize;
  let base = w + size + w;
  (0..size / (2 * w))
    .map(|i| {
      let entry = w + i * 2 * w;
      let start = base + read(entry) as usize;
      let len = data[start..].iter().position(|&b| b == 0).unwrap();
      (
        String::from_utf8_lossy(&data[start..start + len]).into_owned(),
        read(entry + w),
      )
    })
    .collect()
}

fn archive_symbols(lib: &Path) -> Vec<(String, String)> {
  let mut f = File::open(lib).unwrap();
  let mut magic = [0u8; 8];
  f.read_exact(&mut magic).unwrap();
  assert_eq!(&magic, b"!<arch>\n");

  let first = Member::at(&mut f, 8, None);
  let (symbols, mut pos) = match first.name.as_str() {
    "/" => (coff_symbols(&first.data), first.next),
    name if name.starts_with("__.SYMDEF") => {
      (bsd_symbols(&first.data, name.contains("_64")), first.next)
    }
    _ => unreachable!("missing symbol table?"),
  };

  let mut string_table = None;
  // coff sucks and has 2 members named /
  for _ in 0..2 {
    let m = Member::at(&mut f, pos, None);
    if m.name == "//" {
      string_table = Some(m.data);
      break;
    }
    pos = m.next;
  }

  let mut member_names: HashMap<u64, String> = HashMap::new();
  symbols
    .into_iter()
    .map(|(sym, off)| {
      let name = member_names
        .entry(off)
        .or_insert_with(|| {
          Member::at(&mut f, off, string_table.as_deref()).name
        })
        .clone();
      (sym, name)
    })
    .collect()
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
  for e in fs::read_dir(dir).unwrap() {
    let p = e.unwrap().path();
    if p.is_dir() {
      walk(&p, out);
    } else if p.extension().is_some_and(|e| e == "rs") {
      out.push(p);
    }
  }
}

pub fn collect_c_symbols(
  lib: &Path,
  src_dir: &Path,
  binding: &Path,
  is_apple: bool,
) -> Vec<String> {
  let mut idents = HashSet::new();
  let mut files = Vec::new();
  walk(src_dir, &mut files);
  files.push(binding.to_path_buf());

  // just include anything referenced in rust since i dont have a better way of filtering the required symbols ¯\_(ツ)_/¯
  for f in files {
    let text = fs::read_to_string(&f).unwrap();
    for t in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
      if !t.is_empty() {
        idents.insert(t.to_string());
      }
    }
  }

  let mut out = BTreeSet::new();
  for (sym, member) in archive_symbols(lib) {
    let sym = match is_apple {
      true => match sym.strip_prefix('_') {
        Some(sym) => sym.to_string(),
        None => continue,
      },
      false => sym,
    };

    let is_binding =
      member.ends_with("binding.obj") || member.ends_with("binding.o");
    let is_c_symbol = !sym.starts_with('?') && !sym.starts_with('_');
    if is_c_symbol
      && ((is_binding && sym.starts_with("v8__")) || idents.contains(&sym))
    {
      out.insert(sym);
    }
  }
  out.into_iter().collect()
}
