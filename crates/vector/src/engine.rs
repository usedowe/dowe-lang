use crate::error::{VectorError, VectorResult};
use crate::names::{validate_database_name, validate_embedding_id};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::{Mutex, OnceLock};

const FORMAT_VERSION: u32 = 1;
const MAX_DIMENSIONS: usize = 65_536;
const MAX_LIMIT: usize = 1_000;

static WRITE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static TEMP_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Embedding {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorMatch {
    pub id: String,
    pub score: f32,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VectorUpsert {
    pub id: String,
    pub dimensions: usize,
    pub created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VectorInspection {
    pub name: String,
    pub dimensions: Option<usize>,
    pub embeddings: usize,
}

#[derive(Debug, Clone)]
pub struct VectorDatabase {
    root: PathBuf,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Manifest {
    version: u32,
    name: String,
    dimensions: Option<usize>,
}

pub fn vector_root(project_root: &Path) -> PathBuf {
    project_root.join(".dowe").join("vector")
}

pub fn init_database(project_root: &Path, name: &str) -> VectorResult<VectorDatabase> {
    validate_database_name(name)?;
    let database = VectorDatabase {
        root: vector_root(project_root).join(name),
        name: name.to_string(),
    };
    fs::create_dir_all(database.entries_root())?;
    if !database.manifest_path().exists() {
        database.write_manifest(&Manifest {
            version: FORMAT_VERSION,
            name: name.to_string(),
            dimensions: None,
        })?;
    } else {
        database.read_manifest()?;
    }
    Ok(database)
}

pub fn open_database(
    project_root: &Path,
    name: &str,
    create: bool,
) -> VectorResult<VectorDatabase> {
    if create {
        return init_database(project_root, name);
    }
    validate_database_name(name)?;
    let database = VectorDatabase {
        root: vector_root(project_root).join(name),
        name: name.to_string(),
    };
    if !database.manifest_path().exists() {
        return Err(VectorError::NotFound(format!(
            "Vector database `{name}` was not found"
        )));
    }
    database.read_manifest()?;
    Ok(database)
}

pub fn list_databases(project_root: &Path) -> VectorResult<Vec<String>> {
    let root = vector_root(project_root);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if name == "_auth" || validate_database_name(&name).is_err() {
            continue;
        }
        if entry.path().join("manifest.json").is_file() {
            names.push(name);
        }
    }
    names.sort();
    Ok(names)
}

impl VectorDatabase {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn upsert(
        &self,
        id: &str,
        vector: Vec<f32>,
        metadata: Value,
    ) -> VectorResult<VectorUpsert> {
        validate_embedding_id(id)?;
        validate_vector(&vector)?;
        validate_metadata(&metadata)?;
        let _guard = write_guard()?;
        let mut manifest = self.read_manifest()?;
        match manifest.dimensions {
            Some(dimensions) if dimensions != vector.len() => {
                return Err(VectorError::InvalidRequest(format!(
                    "Vector database `{}` requires {dimensions} dimensions, received {}",
                    self.name,
                    vector.len()
                )));
            }
            None => {
                manifest.dimensions = Some(vector.len());
                self.write_manifest(&manifest)?;
            }
            _ => {}
        }
        let path = self.entry_path(id);
        let created = !path.exists();
        write_json(
            &path,
            &Embedding {
                id: id.to_string(),
                vector,
                metadata,
            },
        )?;
        Ok(VectorUpsert {
            id: id.to_string(),
            dimensions: manifest.dimensions.unwrap_or_default(),
            created,
        })
    }

    pub fn search(
        &self,
        vector: &[f32],
        limit: usize,
        min_score: f32,
        filter: Option<&Value>,
    ) -> VectorResult<Vec<VectorMatch>> {
        validate_vector(vector)?;
        validate_limit(limit)?;
        if !min_score.is_finite() || !(-1.0..=1.0).contains(&min_score) {
            return Err(VectorError::InvalidRequest(
                "Vector search minScore must be between -1 and 1".to_string(),
            ));
        }
        validate_filter(filter)?;
        let manifest = self.read_manifest()?;
        match manifest.dimensions {
            Some(dimensions) if dimensions != vector.len() => {
                return Err(VectorError::InvalidRequest(format!(
                    "Vector database `{}` requires {dimensions} dimensions, received {}",
                    self.name,
                    vector.len()
                )));
            }
            None => return Ok(Vec::new()),
            _ => {}
        }
        let query_norm = magnitude(vector);
        let mut matches = self
            .read_all()?
            .into_iter()
            .filter(|embedding| metadata_matches(&embedding.metadata, filter))
            .filter_map(|embedding| {
                let score =
                    dot(vector, &embedding.vector) / (query_norm * magnitude(&embedding.vector));
                (score >= min_score).then_some(VectorMatch {
                    id: embedding.id,
                    score,
                    metadata: embedding.metadata,
                })
            })
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
                .then(left.id.cmp(&right.id))
        });
        matches.truncate(limit);
        Ok(matches)
    }

    pub fn read(&self, id: &str) -> VectorResult<Option<Embedding>> {
        validate_embedding_id(id)?;
        let path = self.entry_path(id);
        if !path.exists() {
            return Ok(None);
        }
        read_json(&path).map(Some)
    }

    pub fn delete(&self, id: &str) -> VectorResult<bool> {
        validate_embedding_id(id)?;
        let _guard = write_guard()?;
        let path = self.entry_path(id);
        if !path.exists() {
            return Ok(false);
        }
        fs::remove_file(path)?;
        Ok(true)
    }

    pub fn list(&self, limit: usize, filter: Option<&Value>) -> VectorResult<Vec<Embedding>> {
        validate_limit(limit)?;
        validate_filter(filter)?;
        let mut embeddings = self
            .read_all()?
            .into_iter()
            .filter(|embedding| metadata_matches(&embedding.metadata, filter))
            .collect::<Vec<_>>();
        embeddings.sort_by(|left, right| left.id.cmp(&right.id));
        embeddings.truncate(limit);
        Ok(embeddings)
    }

    pub fn inspect(&self) -> VectorResult<VectorInspection> {
        let manifest = self.read_manifest()?;
        Ok(VectorInspection {
            name: self.name.clone(),
            dimensions: manifest.dimensions,
            embeddings: self.read_all()?.len(),
        })
    }

    fn read_all(&self) -> VectorResult<Vec<Embedding>> {
        let mut paths = fs::read_dir(self.entries_root())?
            .filter_map(Result::ok)
            .filter_map(|entry| {
                entry
                    .file_type()
                    .ok()
                    .filter(|kind| kind.is_file())
                    .map(|_| entry.path())
            })
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
            .collect::<Vec<_>>();
        paths.sort();
        paths
            .into_iter()
            .map(|path| read_json::<Embedding>(&path))
            .collect()
    }

    fn entries_root(&self) -> PathBuf {
        self.root.join("entries")
    }

    fn entry_path(&self, id: &str) -> PathBuf {
        self.entries_root().join(format!("{}.json", digest(id)))
    }

    fn manifest_path(&self) -> PathBuf {
        self.root.join("manifest.json")
    }

    fn read_manifest(&self) -> VectorResult<Manifest> {
        let manifest: Manifest = read_json(&self.manifest_path())?;
        if manifest.version != FORMAT_VERSION || manifest.name != self.name {
            return Err(VectorError::Corruption(format!(
                "Vector database `{}` has an incompatible manifest",
                self.name
            )));
        }
        Ok(manifest)
    }

    fn write_manifest(&self, manifest: &Manifest) -> VectorResult<()> {
        write_json(&self.manifest_path(), manifest)
    }
}

fn validate_vector(vector: &[f32]) -> VectorResult<()> {
    if vector.is_empty() || vector.len() > MAX_DIMENSIONS {
        return Err(VectorError::InvalidRequest(format!(
            "Vector must contain between 1 and {MAX_DIMENSIONS} dimensions"
        )));
    }
    if vector.iter().any(|value| !value.is_finite()) {
        return Err(VectorError::InvalidRequest(
            "Vector dimensions must be finite numbers".to_string(),
        ));
    }
    let magnitude = magnitude(vector);
    if !magnitude.is_finite() || magnitude == 0.0 {
        return Err(VectorError::InvalidRequest(
            "Vector magnitude must be finite and greater than zero".to_string(),
        ));
    }
    Ok(())
}

fn validate_metadata(metadata: &Value) -> VectorResult<()> {
    if metadata.is_object() {
        Ok(())
    } else {
        Err(VectorError::InvalidRequest(
            "Vector metadata must be an object".to_string(),
        ))
    }
}

fn validate_filter(filter: Option<&Value>) -> VectorResult<()> {
    match filter {
        Some(value) => validate_metadata(value),
        None => Ok(()),
    }
}

fn validate_limit(limit: usize) -> VectorResult<()> {
    if (1..=MAX_LIMIT).contains(&limit) {
        Ok(())
    } else {
        Err(VectorError::InvalidRequest(format!(
            "Vector limit must be between 1 and {MAX_LIMIT}"
        )))
    }
}

fn metadata_matches(metadata: &Value, filter: Option<&Value>) -> bool {
    let Some(Value::Object(filter)) = filter else {
        return true;
    };
    let Some(metadata) = metadata.as_object() else {
        return false;
    };
    filter
        .iter()
        .all(|(key, expected)| metadata.get(key) == Some(expected))
}

fn dot(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

fn magnitude(vector: &[f32]) -> f32 {
    dot(vector, vector).sqrt()
}

fn digest(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn write_guard() -> VectorResult<std::sync::MutexGuard<'static, ()>> {
    WRITE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| VectorError::DurabilityError("Vector write lock failed".to_string()))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> VectorResult<T> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(|error| VectorError::Corruption(error.to_string()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> VectorResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| VectorError::DurabilityError("Vector path has no parent".to_string()))?;
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(
        ".vector-{}.tmp",
        TEMP_ID.fetch_add(1, AtomicOrdering::Relaxed)
    ));
    let bytes = serde_json::to_vec(value)?;
    let mut file = File::create(&temp)?;
    file.write_all(&bytes)?;
    file.sync_all()
        .map_err(|error| VectorError::DurabilityError(error.to_string()))?;
    fs::rename(temp, path)?;
    Ok(())
}

pub(crate) fn object() -> Value {
    Value::Object(Map::new())
}
