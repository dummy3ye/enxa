//! Validated on-disk form of enso's packed tables.
//!
//! Owns: file layout, header checks, checksum, path selection and the in-process fallback.
//! Must NOT: change table contents, run a solve, or parse cube/CLI arguments.
//!
//! Layout (little-endian): 8-byte magic, u32 version, seven u32 sizes, u64 FNV-1a
//! checksum of the payload, then the packed entries and rank lookup arrays. A file is
//! used only when every field matches the compiled-in sizes and the checksum matches.
//! Anything else is reported on stderr and rebuilt in-process.

use crate::stages::{orient, r#final, slicecorner, tetrad};
use crate::subgroup::EDGE_RANK;
use crate::tables::Tables;
use cubecore::N_CPERM;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const MAGIC: &[u8; 8] = b"ENSOTBL1";
const VERSION: u32 = 1;
const HEADER_LEN: usize = 48;
const CHECKSUM_OFF: usize = 40;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x100_0000_01b3;

/// Where a startup attempt got its tables.
pub(crate) enum Source {
    Loaded(PathBuf),
    Built(Option<PathBuf>),
}

impl Source {
    pub(crate) fn path(&self) -> Option<&Path> {
        match self {
            Source::Loaded(p) | Source::Built(Some(p)) => Some(p),
            Source::Built(None) => None,
        }
    }

    pub(crate) fn is_loaded(&self) -> bool {
        matches!(self, Source::Loaded(_))
    }
}

/// Tables plus the startup timing and source, kept separate from solve latency.
pub(crate) struct Startup {
    pub(crate) tables: Tables,
    pub(crate) elapsed: Duration,
    pub(crate) source: Source,
}

#[derive(Clone, Copy)]
struct Sizes {
    st1: usize,
    st2: usize,
    st3: usize,
    st4: usize,
    corner_class: usize,
    cp_idx: usize,
    ep_idx: usize,
}

fn expected_sizes() -> Sizes {
    Sizes {
        st1: orient::N,
        st2: slicecorner::N,
        st3: tetrad::N,
        st4: r#final::N,
        corner_class: N_CPERM,
        cp_idx: N_CPERM,
        ep_idx: EDGE_RANK,
    }
}

fn payload_len(s: &Sizes) -> Option<usize> {
    let entries = s
        .st1
        .checked_add(s.st2)?
        .checked_add(s.st3)?
        .checked_add(s.st4)?;
    let entries = entries.checked_mul(4)?;
    let arrays = s
        .corner_class
        .checked_mul(1)?
        .checked_add(s.cp_idx.checked_mul(2)?)?
        .checked_add(s.ep_idx.checked_mul(2)?)?;
    entries.checked_add(arrays)
}

fn checksum(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

fn path() -> PathBuf {
    if let Some(p) = std::env::var_os("ENSO_TABLES") {
        return PathBuf::from(p);
    }
    let dir = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_else(std::env::temp_dir)
        .join("enxa");
    dir.join(format!("enso-tables-v{VERSION}.bin"))
}

/// Try the cache file, then build in-process and write it for next time.
pub(crate) fn load() -> Startup {
    let started = Instant::now();
    let path = path();
    match fs::read(&path) {
        Ok(bytes) => match decode(&bytes) {
            Ok(tables) => {
                return Startup {
                    tables,
                    elapsed: started.elapsed(),
                    source: Source::Loaded(path),
                }
            }
            Err(reason) => eprintln!(
                "enso: ignoring table file {}: {reason}; rebuilding",
                path.display()
            ),
        },
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => eprintln!(
            "enso: cannot read table file {}: {e}; rebuilding",
            path.display()
        ),
    }
    let tables = Tables::build();
    let source = match save(&tables, &path) {
        Ok(()) => Source::Built(Some(path)),
        Err(e) => {
            eprintln!(
                "enso: cannot write table file {}: {e}; using in-memory tables",
                path.display()
            );
            Source::Built(None)
        }
    };
    Startup {
        tables,
        elapsed: started.elapsed(),
        source,
    }
}

/// Force a rebuild and rewrite; used by `enso --build-tables`.
pub(crate) fn force_build() -> Startup {
    let started = Instant::now();
    let path = path();
    let tables = Tables::build();
    let source = match save(&tables, &path) {
        Ok(()) => Source::Built(Some(path)),
        Err(e) => {
            eprintln!(
                "enso: cannot write table file {}: {e}; using in-memory tables",
                path.display()
            );
            Source::Built(None)
        }
    };
    Startup {
        tables,
        elapsed: started.elapsed(),
        source,
    }
}

fn save(tables: &Tables, path: &Path) -> io::Result<()> {
    let bytes = encode(tables);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!("bin.tmp{}", std::process::id()));
    fs::write(&tmp, &bytes)?;
    fs::rename(&tmp, path)
}

/// Serialize without changing the in-process tables (also the fallback test oracle).
pub(crate) fn encode(tables: &Tables) -> Vec<u8> {
    let s = expected_sizes();
    let mut out = Vec::with_capacity(HEADER_LEN + payload_len(&s).expect("sizes fit"));
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    for n in [
        s.st1,
        s.st2,
        s.st3,
        s.st4,
        s.corner_class,
        s.cp_idx,
        s.ep_idx,
    ] {
        out.extend_from_slice(&(n as u32).to_le_bytes());
    }
    out.extend_from_slice(&[0u8; 8]); // checksum placeholder
    debug_assert_eq!(out.len(), HEADER_LEN);

    for entry in [
        tables.st1.entry(),
        tables.st2.entry(),
        tables.st3.entry(),
        tables.st4.entry(),
    ] {
        for e in entry {
            out.extend_from_slice(&e.to_le_bytes());
        }
    }
    out.extend_from_slice(&tables.corner_class);
    for v in &tables.cp_idx {
        out.extend_from_slice(&v.to_le_bytes());
    }
    for v in &tables.ep_idx {
        out.extend_from_slice(&v.to_le_bytes());
    }
    let sum = checksum(&out[HEADER_LEN..]);
    out[CHECKSUM_OFF..HEADER_LEN].copy_from_slice(&sum.to_le_bytes());
    out
}

/// Decode only files that match the compiled-in format exactly.
pub(crate) fn decode(bytes: &[u8]) -> Result<Tables, &'static str> {
    if bytes.len() < HEADER_LEN {
        return Err("file is shorter than the header");
    }
    if &bytes[..8] != MAGIC {
        return Err("wrong magic");
    }
    if read_u32(bytes, 8)? != VERSION {
        return Err("unsupported format version");
    }
    let s = expected_sizes();
    let sizes = [
        read_u32(bytes, 12)?,
        read_u32(bytes, 16)?,
        read_u32(bytes, 20)?,
        read_u32(bytes, 24)?,
        read_u32(bytes, 28)?,
        read_u32(bytes, 32)?,
        read_u32(bytes, 36)?,
    ];
    let expected = [
        s.st1,
        s.st2,
        s.st3,
        s.st4,
        s.corner_class,
        s.cp_idx,
        s.ep_idx,
    ];
    for (got, want) in sizes.iter().zip(expected) {
        if *got as usize != want {
            return Err("size does not match this build");
        }
    }
    let want_sum = read_u64(bytes, CHECKSUM_OFF)?;
    let len = HEADER_LEN + payload_len(&s).ok_or("size overflow")?;
    if bytes.len() != len {
        return Err("payload length does not match the header");
    }
    if checksum(&bytes[HEADER_LEN..]) != want_sum {
        return Err("payload checksum mismatch");
    }

    let mut off = HEADER_LEN;
    let st1 = take_u32s(bytes, &mut off, s.st1)?;
    let st2 = take_u32s(bytes, &mut off, s.st2)?;
    let st3 = take_u32s(bytes, &mut off, s.st3)?;
    let st4 = take_u32s(bytes, &mut off, s.st4)?;
    let corner_class = take(bytes, &mut off, s.corner_class)?.to_vec();
    let cp_idx = take_u16s(bytes, &mut off, s.cp_idx)?;
    let ep_idx = take_u16s(bytes, &mut off, s.ep_idx)?;
    if off != bytes.len() {
        return Err("trailing bytes after payload");
    }
    let goal3 = tetrad::goal(&corner_class);
    Ok(Tables {
        st1: crate::chain::PackedStage::from_entry(st1),
        st2: crate::chain::PackedStage::from_entry(st2),
        st3: crate::chain::PackedStage::from_entry(st3),
        st4: crate::chain::PackedStage::from_entry(st4),
        corner_class,
        cp_idx,
        ep_idx,
        goal3,
    })
}

fn read_u32(bytes: &[u8], off: usize) -> Result<u32, &'static str> {
    let a = bytes.get(off..off + 4).ok_or("truncated header")?;
    Ok(u32::from_le_bytes(a.try_into().expect("4 bytes")))
}

fn read_u64(bytes: &[u8], off: usize) -> Result<u64, &'static str> {
    let a = bytes.get(off..off + 8).ok_or("truncated header")?;
    Ok(u64::from_le_bytes(a.try_into().expect("8 bytes")))
}

fn take<'a>(bytes: &'a [u8], off: &mut usize, len: usize) -> Result<&'a [u8], &'static str> {
    let end = off.checked_add(len).ok_or("size overflow")?;
    let slice = bytes.get(*off..end).ok_or("truncated payload")?;
    *off = end;
    Ok(slice)
}

fn take_u32s(bytes: &[u8], off: &mut usize, n: usize) -> Result<Vec<u32>, &'static str> {
    let raw = take(bytes, off, n.checked_mul(4).ok_or("size overflow")?)?;
    Ok(raw
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes(c.try_into().expect("4 bytes")))
        .collect())
}

fn take_u16s(bytes: &[u8], off: &mut usize, n: usize) -> Result<Vec<u16>, &'static str> {
    let raw = take(bytes, off, n.checked_mul(2).ok_or("size overflow")?)?;
    Ok(raw
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes(c.try_into().expect("2 bytes")))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tables::tables;

    fn same(a: &Tables, b: &Tables) {
        assert_eq!(a.st1.entry(), b.st1.entry());
        assert_eq!(a.st2.entry(), b.st2.entry());
        assert_eq!(a.st3.entry(), b.st3.entry());
        assert_eq!(a.st4.entry(), b.st4.entry());
        assert_eq!(a.corner_class, b.corner_class);
        assert_eq!(a.cp_idx, b.cp_idx);
        assert_eq!(a.ep_idx, b.ep_idx);
        assert_eq!(a.goal3, b.goal3);
    }

    #[test]
    fn file_round_trip_is_byte_identical() {
        let built = tables();
        let bytes = encode(built);
        let loaded = decode(&bytes).expect("valid file rejected");
        same(built, &loaded);
        assert_eq!(
            bytes.len(),
            HEADER_LEN + payload_len(&expected_sizes()).unwrap()
        );
    }

    #[test]
    fn wrong_magic_version_sizes_and_checksum_are_rejected() {
        let good = encode(tables());
        let mut bad = good.clone();
        bad[0] = b'X';
        assert!(decode(&bad).is_err());
        let mut bad = good.clone();
        bad[8] = 9;
        assert!(decode(&bad).is_err());
        let mut bad = good.clone();
        bad[12] = 7;
        assert!(decode(&bad).is_err());
        let mut bad = good.clone();
        let last = bad.len() - 1;
        bad[last] ^= 1;
        assert!(decode(&bad).is_err());
        assert!(decode(&good[..HEADER_LEN - 1]).is_err());
    }
}
