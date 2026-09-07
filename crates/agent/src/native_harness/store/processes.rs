use super::*;

struct Lease {
    marker: PathBuf,
    value: Value,
    _lock: AuthFileLock,
}
impl super::super::ShellLease for Lease {
    fn started(&mut self, pid: Option<u32>) -> AgentResult<()> {
        self.value["system_pid"] = serde_json::json!(pid);
        write_private_json(&self.marker, &self.value)
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.marker);
    }
}

impl HarnessStore {
    pub(crate) fn acquire_shell(
        &self,
        resource: &str,
        session: &str,
    ) -> AgentResult<Box<dyn super::super::ShellLease + Send>> {
        let id = digest(resource.as_bytes());
        let directory = self.directory().join("processes");
        reject_symlink_ancestors(&directory)?;
        fs::create_dir_all(&directory)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
        }
        let marker = directory.join(format!("{id}.json"));
        reject_symlink_ancestors(&marker)?;
        let lock = AuthFileLock::acquire(&marker.with_extension("lock")).map_err(|_| {
            AgentError::new(
                "this shell/resource is already owned by another active task; inspect /processes",
            )
        })?;
        if marker.exists() {
            return Err(AgentError::new(
                "interrupted shell ownership requires inspection; use /processes before explicitly clearing its stale record",
            ));
        }
        let value = serde_json::json!({"id":id,"session":session,"resource":resource,"created":now(),"system_pid":null});
        write_private_json(&marker, &value)?;
        Ok(Box::new(Lease {
            marker,
            value,
            _lock: lock,
        }))
    }

    pub fn processes(&self) -> AgentResult<Vec<Value>> {
        let directory = self.directory().join("processes");
        reject_symlink_ancestors(&directory)?;
        if !directory.exists() {
            return Ok(vec![]);
        }
        let mut records = Vec::new();
        for entry in fs::read_dir(directory)? {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            reject_symlink_ancestors(&path)?;
            let metadata = match fs::metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error.into()),
            };
            if metadata.len() > 8192 {
                return Err(AgentError::new("invalid process ownership record"));
            }
            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error.into()),
            };
            let mut record: Value = serde_json::from_slice(&bytes)?;
            record["state"] =
                serde_json::json!(
                    if AuthFileLock::acquire(&path.with_extension("lock")).is_ok() {
                        "requires_inspection"
                    } else {
                        "active"
                    }
                );
            Redactor::for_project(&self.root).value(&mut record);
            records.push(record);
        }
        records.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        Ok(records)
    }

    pub fn clear_inactive_process(&self, id: &str) -> AgentResult<()> {
        if id.len() != 64 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(AgentError::new("invalid process ownership id"));
        }
        let path = self
            .directory()
            .join("processes")
            .join(format!("{id}.json"));
        reject_symlink_ancestors(&path)?;
        let _lock = AuthFileLock::acquire(&path.with_extension("lock"))
            .map_err(|_| AgentError::new("cannot clear ownership of an active task"))?;
        fs::remove_file(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ownership_prevents_duplicates_and_stale_records_require_inspection() {
        let home = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        let store = HarnessStore::new(home.path(), root.path()).unwrap();
        let lease = store.acquire_shell("resource:dev:web", "first").unwrap();
        assert!(store.acquire_shell("resource:dev:web", "second").is_err());
        let records = store.processes().unwrap();
        assert_eq!(records[0]["state"], "active");
        let id = records[0]["id"].as_str().unwrap();
        assert!(store.clear_inactive_process(id).is_err());
        let path = store
            .directory()
            .join("processes")
            .join(format!("{id}.json"));
        let bytes = fs::read(&path).unwrap();
        drop(lease);
        assert!(store.processes().unwrap().is_empty());
        fs::write(path, bytes).unwrap();
        assert!(store.acquire_shell("resource:dev:web", "second").is_err());
        assert_eq!(
            store.processes().unwrap()[0]["state"],
            "requires_inspection"
        );
        store.clear_inactive_process(id).unwrap();
        assert!(store.acquire_shell("resource:dev:web", "second").is_ok());
    }
}
