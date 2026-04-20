//! Binary serialisation and deserialisation of a `BytecodeModule`.
//!
//! On-disk format (all integers are little-endian):
//!
//! ```text
//! [magic 6B]           b"SELVR\x01"
//! [version 2B]         u8 major, u8 minor
//! [source len 2B]      u16
//! [source bytes]       UTF-8
//! [src hash 32B]       SHA-256 (or zeros)
//! [pool count 2B]      u16 — number of entries in the constant pool
//!   per entry:
//!     [tag 1B]         0=Str, 1=Name
//!     [len 2B]         u16
//!     [bytes]          UTF-8
//! [fn count 2B]        u16
//!   per fn:
//!     [name_idx 2B]    u16
//!     [param_count 1B] u8
//!     [local_count 2B] u16
//!     [flags 1B]       bit0=is_export, bit1=is_async
//!     [code_len 4B]    u32
//!     [code bytes]
//! ```

use crate::module::{BytecodeModule, BcFn, ConstPool, ConstValue, MAGIC, VERSION};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("bytecode too large")]
    TooLarge,
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("invalid magic bytes")]
    BadMagic,
    #[error("unsupported version {0}.{1}")]
    UnsupportedVersion(u8, u8),
    #[error("invalid UTF-8")]
    InvalidUtf8,
}

// ── Encoder ───────────────────────────────────────────────────────────────────

pub fn encode(m: &BytecodeModule) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::new();

    // Magic + version
    buf.extend_from_slice(MAGIC);
    buf.push(VERSION.0);
    buf.push(VERSION.1);

    // Source name
    write_str(&mut buf, m.source.as_str());

    // Source hash
    buf.extend_from_slice(&m.src_hash);

    // Constant pool
    let pool_len = m.const_pool.entries.len() as u16;
    buf.extend_from_slice(&pool_len.to_le_bytes());
    for entry in &m.const_pool.entries {
        match entry {
            ConstValue::Str(s)  => { buf.push(0); write_str(&mut buf, s); }
            ConstValue::Name(s) => { buf.push(1); write_str(&mut buf, s); }
        }
    }

    // Functions
    let fn_len = m.fns.len() as u16;
    buf.extend_from_slice(&fn_len.to_le_bytes());
    for f in &m.fns {
        buf.extend_from_slice(&f.name_idx.to_le_bytes());
        buf.push(f.param_count);
        buf.extend_from_slice(&f.local_count.to_le_bytes());
        let flags: u8 = (f.is_export as u8) | ((f.is_async as u8) << 1);
        buf.push(flags);
        let code_len = f.code.len() as u32;
        buf.extend_from_slice(&code_len.to_le_bytes());
        buf.extend_from_slice(&f.code);
    }

    Ok(buf)
}

fn write_str(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    let len = bytes.len() as u16;
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(bytes);
}

// ── Decoder ───────────────────────────────────────────────────────────────────

pub fn decode(data: &[u8]) -> Result<BytecodeModule, DecodeError> {
    let mut r = Reader::new(data);

    // Magic
    let magic = r.read_bytes(6)?;
    if magic != MAGIC.as_slice() { return Err(DecodeError::BadMagic); }

    // Version
    let major = r.read_u8()?;
    let minor = r.read_u8()?;
    if major != VERSION.0 { return Err(DecodeError::UnsupportedVersion(major, minor)); }

    // Source name
    let source = r.read_str()?;

    // Hash
    let hash_bytes = r.read_bytes(32)?;
    let mut src_hash = [0u8; 32];
    src_hash.copy_from_slice(hash_bytes);

    // Constant pool
    let pool_count = r.read_u16()? as usize;
    let mut pool   = ConstPool::new();
    for _ in 0..pool_count {
        let tag = r.read_u8()?;
        let s   = smol_str::SmolStr::new(r.read_str()?);
        pool.entries.push(if tag == 0 { ConstValue::Str(s) } else { ConstValue::Name(s) });
    }

    // Functions
    let fn_count = r.read_u16()? as usize;
    let mut fns  = Vec::with_capacity(fn_count);
    for _ in 0..fn_count {
        let name_idx    = r.read_u16()?;
        let param_count = r.read_u8()?;
        let local_count = r.read_u16()?;
        let flags       = r.read_u8()?;
        let is_export   = (flags & 1) != 0;
        let is_async    = (flags & 2) != 0;
        let code_len    = r.read_u32()? as usize;
        let code        = r.read_bytes(code_len)?.to_vec();
        fns.push(BcFn { name_idx, param_count, local_count, code, is_export, is_async });
    }

    Ok(BytecodeModule { const_pool: pool, fns, source: smol_str::SmolStr::new(source), src_hash })
}

// ── Byte reader ───────────────────────────────────────────────────────────────

struct Reader<'a> { data: &'a [u8], pos: usize }

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self { Self { data, pos: 0 } }

    fn read_u8(&mut self) -> Result<u8, DecodeError> {
        let &b = self.data.get(self.pos).ok_or(DecodeError::UnexpectedEof)?;
        self.pos += 1;
        Ok(b)
    }

    fn read_u16(&mut self) -> Result<u16, DecodeError> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    fn read_u32(&mut self) -> Result<u32, DecodeError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        if self.pos + n > self.data.len() { return Err(DecodeError::UnexpectedEof); }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    fn read_str(&mut self) -> Result<&'a str, DecodeError> {
        let len = self.read_u16()? as usize;
        let bytes = self.read_bytes(len)?;
        std::str::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)
    }
}
