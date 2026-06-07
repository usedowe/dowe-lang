use crate::error::{StoreError, StoreResult};
use crate::value::StoreValue;
use std::collections::BTreeMap;

const VALUE_NULL: u8 = 0;
const VALUE_BOOL: u8 = 1;
const VALUE_INT: u8 = 2;
const VALUE_UINT: u8 = 3;
const VALUE_FLOAT: u8 = 4;
const VALUE_DECIMAL: u8 = 5;
const VALUE_STRING: u8 = 6;
const VALUE_BYTES: u8 = 7;
const VALUE_TIMESTAMP: u8 = 8;
const VALUE_ULID: u8 = 9;
const VALUE_JSON: u8 = 10;
const VALUE_DSF: u8 = 11;

pub struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn bytes(&mut self, value: &[u8]) {
        self.bytes.extend_from_slice(value);
    }

    pub fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn i64(&mut self, value: i64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn f64(&mut self, value: f64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn string(&mut self, value: &str) {
        self.u32(value.len() as u32);
        self.bytes(value.as_bytes());
    }

    pub fn raw(&mut self, value: &[u8]) {
        self.u32(value.len() as u32);
        self.bytes(value);
    }
}

pub struct Reader<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0 }
    }

    pub fn is_done(&self) -> bool {
        self.index == self.bytes.len()
    }

    pub fn magic(&mut self, expected: &[u8]) -> StoreResult<()> {
        let actual = self.take(expected.len())?;
        if actual != expected {
            return Err(StoreError::Corruption(
                "invalid Store file magic".to_string(),
            ));
        }
        Ok(())
    }

    pub fn u8(&mut self) -> StoreResult<u8> {
        Ok(self.take(1)?[0])
    }

    pub fn u32(&mut self) -> StoreResult<u32> {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(self.take(4)?);
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn u64(&mut self) -> StoreResult<u64> {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(self.take(8)?);
        Ok(u64::from_le_bytes(bytes))
    }

    pub fn i64(&mut self) -> StoreResult<i64> {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(self.take(8)?);
        Ok(i64::from_le_bytes(bytes))
    }

    pub fn f64(&mut self) -> StoreResult<f64> {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(self.take(8)?);
        Ok(f64::from_le_bytes(bytes))
    }

    pub fn string(&mut self) -> StoreResult<String> {
        let length = self.u32()? as usize;
        let bytes = self.take(length)?;
        String::from_utf8(bytes.to_vec())
            .map_err(|_| StoreError::Corruption("invalid UTF-8 in Store file".to_string()))
    }

    pub fn raw(&mut self) -> StoreResult<Vec<u8>> {
        let length = self.u32()? as usize;
        Ok(self.take(length)?.to_vec())
    }

    fn take(&mut self, length: usize) -> StoreResult<&'a [u8]> {
        let end = self.index.saturating_add(length);
        if end > self.bytes.len() {
            return Err(StoreError::Corruption(
                "Store file ended before expected data".to_string(),
            ));
        }
        let slice = &self.bytes[self.index..end];
        self.index = end;
        Ok(slice)
    }
}

pub fn encode_record(record: &BTreeMap<String, StoreValue>) -> StoreResult<Vec<u8>> {
    let mut writer = Writer::new();
    writer.u32(record.len() as u32);
    for (key, value) in record {
        writer.string(key);
        encode_value(&mut writer, value)?;
    }
    Ok(writer.into_bytes())
}

pub fn decode_record(bytes: &[u8]) -> StoreResult<BTreeMap<String, StoreValue>> {
    let mut reader = Reader::new(bytes);
    let count = reader.u32()?;
    let mut record = BTreeMap::new();
    for _ in 0..count {
        let key = reader.string()?;
        let value = decode_value(&mut reader)?;
        record.insert(key, value);
    }
    if !reader.is_done() {
        return Err(StoreError::Corruption(
            "record contains trailing bytes".to_string(),
        ));
    }
    Ok(record)
}

fn encode_value(writer: &mut Writer, value: &StoreValue) -> StoreResult<()> {
    match value {
        StoreValue::Null => writer.u8(VALUE_NULL),
        StoreValue::Bool(value) => {
            writer.u8(VALUE_BOOL);
            writer.u8(u8::from(*value));
        }
        StoreValue::Int(value) => {
            writer.u8(VALUE_INT);
            writer.i64(*value);
        }
        StoreValue::UInt(value) => {
            writer.u8(VALUE_UINT);
            writer.u64(*value);
        }
        StoreValue::Float(value) => {
            writer.u8(VALUE_FLOAT);
            writer.f64(*value);
        }
        StoreValue::Decimal(value) => {
            writer.u8(VALUE_DECIMAL);
            writer.string(value);
        }
        StoreValue::String(value) => {
            writer.u8(VALUE_STRING);
            writer.string(value);
        }
        StoreValue::Bytes(value) => {
            writer.u8(VALUE_BYTES);
            writer.raw(value);
        }
        StoreValue::Timestamp(value) => {
            writer.u8(VALUE_TIMESTAMP);
            writer.string(value);
        }
        StoreValue::Ulid(value) => {
            writer.u8(VALUE_ULID);
            writer.string(value);
        }
        StoreValue::Json(value) => {
            writer.u8(VALUE_JSON);
            writer.string(&value.to_string());
        }
        StoreValue::Dsf(value) => {
            writer.u8(VALUE_DSF);
            writer.string(value);
        }
    }
    Ok(())
}

fn decode_value(reader: &mut Reader<'_>) -> StoreResult<StoreValue> {
    Ok(match reader.u8()? {
        VALUE_NULL => StoreValue::Null,
        VALUE_BOOL => StoreValue::Bool(reader.u8()? != 0),
        VALUE_INT => StoreValue::Int(reader.i64()?),
        VALUE_UINT => StoreValue::UInt(reader.u64()?),
        VALUE_FLOAT => StoreValue::Float(reader.f64()?),
        VALUE_DECIMAL => StoreValue::Decimal(reader.string()?),
        VALUE_STRING => StoreValue::String(reader.string()?),
        VALUE_BYTES => StoreValue::Bytes(reader.raw()?),
        VALUE_TIMESTAMP => StoreValue::Timestamp(reader.string()?),
        VALUE_ULID => StoreValue::Ulid(reader.string()?),
        VALUE_JSON => StoreValue::Json(serde_json::from_str(&reader.string()?)?),
        VALUE_DSF => StoreValue::Dsf(reader.string()?),
        value => {
            return Err(StoreError::Corruption(format!(
                "unknown Store value tag {value}"
            )));
        }
    })
}
