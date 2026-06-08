use crate::codec::{Reader, Writer};
use crate::error::{KvError, KvResult};
use crate::names::{validate_database_name, validate_key};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

const DATABASE_MAGIC: &[u8] = b"DOWE_KV_DATABASE_V1\n";
const VALUE_MAGIC: &[u8] = b"DOWE_KV_VALUE_V1\n";

type MemoryState = Arc<Mutex<BTreeMap<String, Value>>>;

static REGISTRY: OnceLock<Mutex<HashMap<PathBuf, MemoryState>>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvSetReport {
    pub key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KvInspection {
    pub name: String,
    pub persistent: bool,
    pub memory_keys: usize,
    pub persisted_keys: usize,
    pub keys: Vec<String>,
}

#[derive(Clone)]
pub struct KvDatabase {
    root: PathBuf,
    name: String,
    persist: bool,
    memory: MemoryState,
}

pub fn kv_root(project_root: &Path) -> PathBuf {
    project_root.join(".dowe").join("kv")
}

pub fn init_database(project_root: &Path, name: &str) -> KvResult<()> {
    validate_database_name(name)?;
    let root = database_root(project_root, name);
    fs::create_dir_all(root.join("keys"))?;
    let metadata = root.join("metadata.bin");
    if !metadata.exists() {
        let mut writer = Writer::new();
        writer.bytes(DATABASE_MAGIC);
        writer.string(name);
        let mut file = File::create(metadata)?;
        file.write_all(&writer.into_bytes())?;
        file.sync_all()
            .map_err(|error| KvError::DurabilityError(error.to_string()))?;
    }
    Ok(())
}

pub fn open_database(project_root: &Path, name: &str, persist: bool) -> KvResult<KvDatabase> {
    validate_database_name(name)?;
    if persist {
        init_database(project_root, name)?;
    }
    let root = database_root(project_root, name);
    let key = registry_key(&root);
    let memory = {
        let mut registry = REGISTRY
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .map_err(|_| KvError::InvalidRequest("KV registry lock failed".to_string()))?;
        registry
            .entry(key)
            .or_insert_with(|| Arc::new(Mutex::new(BTreeMap::new())))
            .clone()
    };
    Ok(KvDatabase {
        root,
        name: name.to_string(),
        persist,
        memory,
    })
}

pub fn list_databases(project_root: &Path) -> KvResult<Vec<String>> {
    let root = kv_root(project_root);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut databases = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let metadata = entry.path().join("metadata.bin");
        if metadata.exists() {
            databases.push(read_metadata(&metadata)?);
        }
    }
    databases.sort();
    Ok(databases)
}

pub fn clear_memory(project_root: &Path, name: &str) -> KvResult<()> {
    validate_database_name(name)?;
    let root = database_root(project_root, name);
    let key = registry_key(&root);
    let registry = REGISTRY
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .map_err(|_| KvError::InvalidRequest("KV registry lock failed".to_string()))?;
    if let Some(memory) = registry.get(&key) {
        memory
            .lock()
            .map_err(|_| KvError::InvalidRequest("KV memory lock failed".to_string()))?
            .clear();
    }
    Ok(())
}

impl KvDatabase {
    pub fn set(&self, key: &str, value: Value) -> KvResult<KvSetReport> {
        validate_key(key)?;
        if self.persist {
            self.write_persisted(key, &value)?;
        }
        self.memory
            .lock()
            .map_err(|_| KvError::InvalidRequest("KV memory lock failed".to_string()))?
            .insert(key.to_string(), value);
        Ok(KvSetReport {
            key: key.to_string(),
        })
    }

    pub fn get(&self, key: &str) -> KvResult<Option<Value>> {
        validate_key(key)?;
        if let Some(value) = self
            .memory
            .lock()
            .map_err(|_| KvError::InvalidRequest("KV memory lock failed".to_string()))?
            .get(key)
            .cloned()
        {
            return Ok(Some(value));
        }
        if !self.persist {
            return Ok(None);
        }
        let Some(value) = self.read_persisted(key)? else {
            return Ok(None);
        };
        self.memory
            .lock()
            .map_err(|_| KvError::InvalidRequest("KV memory lock failed".to_string()))?
            .insert(key.to_string(), value.clone());
        Ok(Some(value))
    }

    pub fn delete(&self, key: &str) -> KvResult<bool> {
        validate_key(key)?;
        let memory_deleted = self
            .memory
            .lock()
            .map_err(|_| KvError::InvalidRequest("KV memory lock failed".to_string()))?
            .remove(key)
            .is_some();
        let persisted_deleted = if self.persist {
            self.delete_persisted(key)?
        } else {
            false
        };
        Ok(memory_deleted || persisted_deleted)
    }

    pub fn clear(&self) -> KvResult<usize> {
        let memory_count = {
            let mut memory = self
                .memory
                .lock()
                .map_err(|_| KvError::InvalidRequest("KV memory lock failed".to_string()))?;
            let count = memory.len();
            memory.clear();
            count
        };
        let persisted_count = if self.persist {
            self.clear_persisted()?
        } else {
            0
        };
        Ok(memory_count.max(persisted_count))
    }

    pub fn keys(&self, prefix: Option<&str>) -> KvResult<Vec<String>> {
        let mut keys = BTreeSet::new();
        for key in self
            .memory
            .lock()
            .map_err(|_| KvError::InvalidRequest("KV memory lock failed".to_string()))?
            .keys()
        {
            if prefix.is_none_or(|prefix| key.starts_with(prefix)) {
                keys.insert(key.clone());
            }
        }
        if self.persist {
            for key in self.persisted_keys()? {
                if prefix.is_none_or(|prefix| key.starts_with(prefix)) {
                    keys.insert(key);
                }
            }
        }
        Ok(keys.into_iter().collect())
    }

    pub fn inspect(&self) -> KvResult<KvInspection> {
        let memory_keys = self
            .memory
            .lock()
            .map_err(|_| KvError::InvalidRequest("KV memory lock failed".to_string()))?
            .len();
        let persisted = if self.root.join("metadata.bin").exists() {
            self.persisted_keys()?
        } else {
            Vec::new()
        };
        Ok(KvInspection {
            name: self.name.clone(),
            persistent: self.persist,
            memory_keys,
            persisted_keys: persisted.len(),
            keys: self.keys(None)?,
        })
    }

    fn write_persisted(&self, key: &str, value: &Value) -> KvResult<()> {
        init_database_from_root(&self.root, &self.name)?;
        let path = self.value_path(key);
        let mut writer = Writer::new();
        writer.bytes(VALUE_MAGIC);
        writer.string(key);
        writer.raw(&serde_json::to_vec(value)?);
        let mut file = File::create(path)?;
        file.write_all(&writer.into_bytes())?;
        file.sync_all()
            .map_err(|error| KvError::DurabilityError(error.to_string()))?;
        Ok(())
    }

    fn read_persisted(&self, key: &str) -> KvResult<Option<Value>> {
        let path = self.value_path(key);
        if !path.exists() {
            return Ok(None);
        }
        let (stored_key, value) = read_value_file(&path)?;
        if stored_key != key {
            return Err(KvError::Corruption(
                "KV value file key does not match requested key".to_string(),
            ));
        }
        Ok(Some(value))
    }

    fn delete_persisted(&self, key: &str) -> KvResult<bool> {
        let path = self.value_path(key);
        if !path.exists() {
            return Ok(false);
        }
        fs::remove_file(path)?;
        Ok(true)
    }

    fn clear_persisted(&self) -> KvResult<usize> {
        let keys_root = self.root.join("keys");
        if !keys_root.exists() {
            return Ok(0);
        }
        let count = fs::read_dir(&keys_root)?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_type()
                    .map(|value| value.is_file())
                    .unwrap_or(false)
            })
            .count();
        fs::remove_dir_all(&keys_root)?;
        fs::create_dir_all(&keys_root)?;
        Ok(count)
    }

    fn persisted_keys(&self) -> KvResult<Vec<String>> {
        let keys_root = self.root.join("keys");
        if !keys_root.exists() {
            return Ok(Vec::new());
        }
        let mut keys = Vec::new();
        for entry in fs::read_dir(keys_root)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            keys.push(read_value_file(&entry.path())?.0);
        }
        keys.sort();
        Ok(keys)
    }

    fn value_path(&self, key: &str) -> PathBuf {
        self.root
            .join("keys")
            .join(format!("{}.bin", key_digest(key)))
    }
}

fn read_value_file(path: &Path) -> KvResult<(String, Value)> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let mut reader = Reader::new(&bytes);
    reader.magic(VALUE_MAGIC)?;
    let key = reader.string()?;
    let value = serde_json::from_slice::<Value>(&reader.raw()?)?;
    if !reader.is_done() {
        return Err(KvError::Corruption(
            "KV value file contains trailing bytes".to_string(),
        ));
    }
    Ok((key, value))
}

fn read_metadata(path: &Path) -> KvResult<String> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let mut reader = Reader::new(&bytes);
    reader.magic(DATABASE_MAGIC)?;
    let name = reader.string()?;
    if !reader.is_done() {
        return Err(KvError::Corruption(
            "KV metadata contains trailing bytes".to_string(),
        ));
    }
    Ok(name)
}

fn init_database_from_root(root: &Path, name: &str) -> KvResult<()> {
    fs::create_dir_all(root.join("keys"))?;
    let metadata = root.join("metadata.bin");
    if !metadata.exists() {
        let mut writer = Writer::new();
        writer.bytes(DATABASE_MAGIC);
        writer.string(name);
        let mut file = File::create(metadata)?;
        file.write_all(&writer.into_bytes())?;
        file.sync_all()
            .map_err(|error| KvError::DurabilityError(error.to_string()))?;
    }
    Ok(())
}

fn database_root(project_root: &Path, name: &str) -> PathBuf {
    kv_root(project_root).join(name)
}

fn registry_key(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn key_digest(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex(&hasher.finalize())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn inspection_json(inspection: &KvInspection) -> Value {
    let mut output = Map::new();
    output.insert("name".to_string(), Value::String(inspection.name.clone()));
    output.insert("persistent".to_string(), Value::Bool(inspection.persistent));
    output.insert(
        "memoryKeys".to_string(),
        Value::Number(inspection.memory_keys.into()),
    );
    output.insert(
        "persistedKeys".to_string(),
        Value::Number(inspection.persisted_keys.into()),
    );
    output.insert(
        "keys".to_string(),
        Value::Array(
            inspection
                .keys
                .iter()
                .map(|value| Value::String(value.clone()))
                .collect(),
        ),
    );
    Value::Object(output)
}
