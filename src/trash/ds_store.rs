use std::fs;
use std::path::Path;

use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct DsStoreEntry {
    pub filename: String,
    pub structure_type: String,
    pub data_type: String,
    pub value: DsStoreValue,
}

#[derive(Debug, Clone)]
pub enum DsStoreValue {
    Bool(bool),
    U32(u32),
    U64(u64),
    FourCharCode(String),
    String(String),
    Blob(Vec<u8>),
}

impl DsStoreValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(value) | Self::FourCharCode(value) => Some(value.as_str()),
            _ => None,
        }
    }
}

pub fn read_entries(path: &Path) -> Result<Vec<DsStoreEntry>> {
    let bytes = fs::read(path)?;
    parse_entries(&bytes)
}

pub fn upsert_trash_entry(path: &Path, trashed_name: &str, original_path: &Path) -> Result<()> {
    let mut entries = read_entries_or_empty(path)?;
    entries.retain(|entry| {
        !(entry.filename == trashed_name
            && matches!(entry.structure_type.as_str(), "ptbL" | "ptbN"))
    });

    let original_name = original_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| Error::message("invalid original file name"))?;
    let parent_path = original_path.parent().unwrap_or(Path::new("/"));

    entries.push(DsStoreEntry {
        filename: trashed_name.to_string(),
        structure_type: "ptbL".to_string(),
        data_type: "ustr".to_string(),
        value: DsStoreValue::String(parent_path.to_string_lossy().into_owned()),
    });
    entries.push(DsStoreEntry {
        filename: trashed_name.to_string(),
        structure_type: "ptbN".to_string(),
        data_type: "ustr".to_string(),
        value: DsStoreValue::String(original_name.to_string()),
    });

    write_entries(path, &entries)
}

pub fn remove_trash_entry(path: &Path, trashed_name: &str) -> Result<()> {
    let mut entries = read_entries_or_empty(path)?;
    entries.retain(|entry| entry.filename != trashed_name);

    if entries.is_empty() {
        match fs::remove_file(path) {
            Ok(()) => return Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) => return Err(err.into()),
        }
    }

    write_entries(path, &entries)
}

pub fn write_entries(path: &Path, entries: &[DsStoreEntry]) -> Result<()> {
    let mut records = entries.to_vec();
    records.sort_by(|left, right| {
        left.filename
            .cmp(&right.filename)
            .then_with(|| left.structure_type.cmp(&right.structure_type))
            .then_with(|| left.data_type.cmp(&right.data_type))
    });

    let leaf_records = serialize_records(&records)?;
    let leaf_block_size = block_size_for(8usize.saturating_add(leaf_records.len()));
    let master_block_size = block_size_for(20);

    let body_prefix_len = 4 + 4 + 256 * 4 + 4 + 1 + 4 + 4;
    let body_offset = 36usize;
    let body_end = body_offset
        .checked_add(body_prefix_len)
        .ok_or_else(|| Error::message("DS_Store body overflow"))?;

    let master_block_offset = align_block_offset(body_end);
    let leaf_block_offset = align_block_offset(
        master_block_offset
            .checked_add(master_block_size)
            .ok_or_else(|| Error::message("DS_Store block overflow"))?,
    );
    let file_len = leaf_block_offset
        .checked_add(leaf_block_size)
        .ok_or_else(|| Error::message("DS_Store size overflow"))?;

    let mut bytes = vec![0u8; file_len];
    write_header(&mut bytes, body_prefix_len)?;
    write_body(
        &mut bytes[body_offset..body_end],
        &[
            encode_block_address(master_block_offset, master_block_size)?,
            encode_block_address(leaf_block_offset, leaf_block_size)?,
        ],
    )?;
    write_master_block(
        &mut bytes[master_block_offset..master_block_offset + master_block_size],
        records.len() as u32,
    );
    write_leaf_block(
        &mut bytes[leaf_block_offset..leaf_block_offset + leaf_block_size],
        &leaf_records,
        records.len() as u32,
    )?;

    fs::write(path, bytes)?;
    Ok(())
}

pub fn parse_entries(bytes: &[u8]) -> Result<Vec<DsStoreEntry>> {
    let mut cursor = Cursor::new(bytes);
    cursor.expect_bytes(&[0, 0, 0, 1], "invalid DS_Store alignment header")?;
    cursor.expect_bytes(b"Bud1", "invalid DS_Store buddy allocator header")?;

    let offset = cursor.read_u32_be()? as usize;
    let len = cursor.read_u32_be()? as usize;
    let _copy_offset = cursor.read_u32_be()?;
    cursor.skip(16)?;

    let body_start = offset
        .checked_add(4)
        .ok_or_else(|| Error::message("invalid DS_Store body offset"))?;
    let body_end = body_start
        .checked_add(len)
        .ok_or_else(|| Error::message("invalid DS_Store body length"))?;
    if body_end > bytes.len() {
        return Err(Error::message("truncated DS_Store body"));
    }

    let body = &bytes[body_start..body_end];
    let mut body_cursor = Cursor::new(body);
    let _num_blocks = body_cursor.read_u32_be()?;
    body_cursor.skip(4)?;

    let mut block_addresses = Vec::with_capacity(256);
    for _ in 0..256 {
        let address_raw = body_cursor.read_u32_be()?;
        block_addresses.push(BlockAddress::new(address_raw));
    }

    let num_directories = body_cursor.read_u32_be()? as usize;
    let mut directories = Vec::with_capacity(num_directories);
    for _ in 0..num_directories {
        let len_name = body_cursor.read_u8()? as usize;
        let name = body_cursor.read_string(len_name)?;
        let block_id = body_cursor.read_u32_be()? as usize;
        directories.push((name, block_id));
    }

    let mut entries = Vec::new();
    for (_, block_id) in directories {
        let root_block_id = parse_master_block(bytes, &block_addresses, block_id)?;
        collect_block_entries(bytes, &block_addresses, root_block_id, &mut entries)?;
    }

    Ok(entries)
}

fn read_entries_or_empty(path: &Path) -> Result<Vec<DsStoreEntry>> {
    match read_entries(path) {
        Ok(entries) => Ok(entries),
        Err(Error::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(err) => Err(err),
    }
}

fn serialize_records(entries: &[DsStoreEntry]) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    for entry in entries {
        write_ustr32(&mut bytes, &entry.filename)?;
        write_fourcc(&mut bytes, &entry.structure_type)?;
        write_fourcc(&mut bytes, &entry.data_type)?;
        write_value(&mut bytes, &entry.data_type, &entry.value)?;
    }
    Ok(bytes)
}

fn write_header(bytes: &mut [u8], body_len: usize) -> Result<()> {
    if bytes.len() < 36 {
        return Err(Error::message("DS_Store buffer too small"));
    }

    bytes[0..4].copy_from_slice(&[0, 0, 0, 1]);
    bytes[4..8].copy_from_slice(b"Bud1");
    bytes[8..12].copy_from_slice(&(32u32).to_be_bytes());
    bytes[12..16].copy_from_slice(&(body_len as u32).to_be_bytes());
    bytes[16..20].copy_from_slice(&0u32.to_be_bytes());
    Ok(())
}

fn write_body(bytes: &mut [u8], block_addresses: &[u32]) -> Result<()> {
    let expected_len = 4 + 4 + 256 * 4 + 4 + 1 + 4 + 4;
    if bytes.len() != expected_len {
        return Err(Error::message("invalid DS_Store body size"));
    }

    let mut cursor = WriteCursor::new(bytes);
    cursor.write_u32_be(block_addresses.len() as u32)?;
    cursor.write_u32_be(0)?;
    for index in 0..256 {
        cursor.write_u32_be(*block_addresses.get(index).unwrap_or(&0))?;
    }
    cursor.write_u32_be(1)?;
    cursor.write_u8(4)?;
    cursor.write_bytes(b"DSDB")?;
    cursor.write_u32_be(0)?;
    Ok(())
}

fn write_master_block(bytes: &mut [u8], record_count: u32) {
    let mut cursor = WriteCursor::new(bytes);
    let _ = cursor.write_u32_be(1);
    let _ = cursor.write_u32_be(0);
    let _ = cursor.write_u32_be(record_count);
    let _ = cursor.write_u32_be(1);
    let _ = cursor.write_u32_be(0);
}

fn write_leaf_block(bytes: &mut [u8], records: &[u8], record_count: u32) -> Result<()> {
    let mut cursor = WriteCursor::new(bytes);
    cursor.write_u32_be(0)?;
    cursor.write_u32_be(record_count)?;
    cursor.write_bytes(records)?;
    Ok(())
}

fn block_size_for(required_len: usize) -> usize {
    let mut size = 32usize;
    while size < required_len {
        size <<= 1;
    }
    size
}

fn align_block_offset(offset: usize) -> usize {
    let remainder = offset % 32;
    if remainder <= 4 {
        offset + (4 - remainder)
    } else {
        offset + (36 - remainder)
    }
}

fn encode_block_address(offset: usize, size: usize) -> Result<u32> {
    if offset < 4 || offset % 32 != 4 {
        return Err(Error::message("invalid DS_Store block offset"));
    }

    let exponent = size.trailing_zeros() as usize;
    if size != (1usize << exponent) || exponent > 31 {
        return Err(Error::message("invalid DS_Store block size"));
    }

    let base = offset - 4;
    let raw = (base as u32)
        .checked_add(exponent as u32)
        .ok_or_else(|| Error::message("invalid DS_Store block address"))?;
    Ok(raw)
}

fn write_value(bytes: &mut Vec<u8>, data_type: &str, value: &DsStoreValue) -> Result<()> {
    match (data_type, value) {
        ("bool", DsStoreValue::Bool(value)) => bytes.push(u8::from(*value)),
        ("long" | "shor", DsStoreValue::U32(value)) => {
            bytes.extend_from_slice(&value.to_be_bytes())
        }
        ("comp" | "dutc", DsStoreValue::U64(value)) => {
            bytes.extend_from_slice(&value.to_be_bytes())
        }
        ("type", DsStoreValue::FourCharCode(value)) => write_fourcc(bytes, value)?,
        ("ustr", DsStoreValue::String(value)) => write_ustr(bytes, value)?,
        ("blob", DsStoreValue::Blob(value)) => {
            bytes.extend_from_slice(&(value.len() as u32).to_be_bytes());
            bytes.extend_from_slice(value);
        }
        _ => {
            return Err(Error::message(format!(
                "unsupported DS_Store value for type {data_type}"
            )));
        }
    }
    Ok(())
}

fn write_fourcc(bytes: &mut Vec<u8>, value: &str) -> Result<()> {
    if value.len() != 4 {
        return Err(Error::message(format!("invalid DS_Store fourcc: {value}")));
    }
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn write_ustr32(bytes: &mut Vec<u8>, value: &str) -> Result<()> {
    let units = value.encode_utf16().collect::<Vec<_>>();
    let len = u32::try_from(units.len()).map_err(|_| Error::message("DS_Store string too long"))?;
    bytes.extend_from_slice(&len.to_be_bytes());
    for unit in units {
        bytes.extend_from_slice(&unit.to_be_bytes());
    }
    Ok(())
}

fn write_ustr(bytes: &mut Vec<u8>, value: &str) -> Result<()> {
    write_ustr32(bytes, value)
}

#[derive(Debug, Clone, Copy)]
struct BlockAddress {
    offset: usize,
    size: usize,
}

impl BlockAddress {
    fn new(address_raw: u32) -> Self {
        let mask = 31u32;
        let offset = ((address_raw & !mask) as usize).saturating_add(4);
        let size = 1usize << (address_raw & mask);
        Self { offset, size }
    }
}

fn parse_master_block(
    bytes: &[u8],
    block_addresses: &[BlockAddress],
    block_id: usize,
) -> Result<usize> {
    let block_address = block_addresses
        .get(block_id)
        .ok_or_else(|| Error::message(format!("invalid DS_Store master block id: {block_id}")))?;
    let end = block_address
        .offset
        .checked_add(block_address.size)
        .ok_or_else(|| Error::message("invalid DS_Store master block range"))?;
    if end > bytes.len() {
        return Err(Error::message("truncated DS_Store master block"));
    }

    let mut cursor = Cursor::new(&bytes[block_address.offset..end]);
    let root_block_id = cursor.read_u32_be()? as usize;
    let _num_internal_nodes = cursor.read_u32_be()?;
    let _num_records = cursor.read_u32_be()?;
    let _num_nodes = cursor.read_u32_be()?;
    let _unused = cursor.read_u32_be()?;
    Ok(root_block_id)
}

fn collect_block_entries(
    bytes: &[u8],
    block_addresses: &[BlockAddress],
    block_id: usize,
    out: &mut Vec<DsStoreEntry>,
) -> Result<()> {
    let block_address = block_addresses
        .get(block_id)
        .ok_or_else(|| Error::message(format!("invalid DS_Store block id: {block_id}")))?;
    let end = block_address
        .offset
        .checked_add(block_address.size)
        .ok_or_else(|| Error::message("invalid DS_Store block range"))?;
    if end > bytes.len() {
        return Err(Error::message("truncated DS_Store block"));
    }

    let mut cursor = Cursor::new(&bytes[block_address.offset..end]);
    let mode = cursor.read_u32_be()? as usize;
    let count = cursor.read_u32_be()? as usize;

    if mode == 0 {
        for _ in 0..count {
            out.push(parse_record(&mut cursor)?);
        }
        return Ok(());
    }

    for _ in 0..count {
        let child_block_id = cursor.read_u32_be()? as usize;
        collect_block_entries(bytes, block_addresses, child_block_id, out)?;
        out.push(parse_record(&mut cursor)?);
    }

    collect_block_entries(bytes, block_addresses, mode, out)?;

    Ok(())
}

fn parse_record(cursor: &mut Cursor<'_>) -> Result<DsStoreEntry> {
    let filename = cursor.read_ustr32()?;
    let structure_type = cursor.read_fourcc()?;
    let data_type = cursor.read_fourcc()?;
    let value = match data_type.as_str() {
        "bool" => DsStoreValue::Bool(cursor.read_u8()? != 0),
        "comp" => DsStoreValue::U64(cursor.read_u64_be()?),
        "dutc" => DsStoreValue::U64(cursor.read_u64_be()?),
        "long" => DsStoreValue::U32(cursor.read_u32_be()?),
        "shor" => DsStoreValue::U32(cursor.read_u32_be()?),
        "type" => DsStoreValue::FourCharCode(cursor.read_fourcc()?),
        "ustr" => DsStoreValue::String(cursor.read_ustr()?),
        "blob" => DsStoreValue::Blob(cursor.read_blob()?),
        other => {
            return Err(Error::message(format!(
                "unsupported DS_Store value type: {other}"
            )));
        }
    };

    Ok(DsStoreEntry {
        filename,
        structure_type,
        data_type,
        value,
    })
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

struct WriteCursor<'a> {
    bytes: &'a mut [u8],
    pos: usize,
}

impl<'a> WriteCursor<'a> {
    fn new(bytes: &'a mut [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn write_u8(&mut self, value: u8) -> Result<()> {
        self.write_bytes(&[value])
    }

    fn write_u32_be(&mut self, value: u32) -> Result<()> {
        self.write_bytes(&value.to_be_bytes())
    }

    fn write_bytes(&mut self, value: &[u8]) -> Result<()> {
        let end = self
            .pos
            .checked_add(value.len())
            .ok_or_else(|| Error::message("truncated DS_Store"))?;
        let target = self
            .bytes
            .get_mut(self.pos..end)
            .ok_or_else(|| Error::message("truncated DS_Store"))?;
        target.copy_from_slice(value);
        self.pos = end;
        Ok(())
    }
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn expect_bytes(&mut self, expected: &[u8], message: &str) -> Result<()> {
        let actual = self.read_bytes(expected.len())?;
        if actual != expected {
            return Err(Error::message(message));
        }
        Ok(())
    }

    fn skip(&mut self, len: usize) -> Result<()> {
        let _ = self.read_bytes(len)?;
        Ok(())
    }

    fn read_u8(&mut self) -> Result<u8> {
        let byte = *self
            .bytes
            .get(self.pos)
            .ok_or_else(|| Error::message("truncated DS_Store"))?;
        self.pos += 1;
        Ok(byte)
    }

    fn read_u32_be(&mut self) -> Result<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u64_be(&mut self) -> Result<u64> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| Error::message("truncated DS_Store"))?;
        let bytes = self
            .bytes
            .get(self.pos..end)
            .ok_or_else(|| Error::message("truncated DS_Store"))?;
        self.pos = end;
        Ok(bytes)
    }

    fn read_fourcc(&mut self) -> Result<String> {
        let bytes = self.read_bytes(4)?;
        Ok(String::from_utf8_lossy(bytes).to_string())
    }

    fn read_string(&mut self, len: usize) -> Result<String> {
        let bytes = self.read_bytes(len)?;
        Ok(String::from_utf8_lossy(bytes).to_string())
    }

    fn read_ustr(&mut self) -> Result<String> {
        let len = self.read_u32_be()? as usize;
        let bytes = self.read_bytes(len.saturating_mul(2))?;
        let units = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        Ok(String::from_utf16_lossy(&units))
    }

    fn read_ustr32(&mut self) -> Result<String> {
        let len = self.read_u32_be()? as usize;
        let bytes = self.read_bytes(len.saturating_mul(2))?;
        let units = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        Ok(String::from_utf16_lossy(&units))
    }

    fn read_blob(&mut self) -> Result<Vec<u8>> {
        let len = self.read_u32_be()? as usize;
        Ok(self.read_bytes(len)?.to_vec())
    }
}
