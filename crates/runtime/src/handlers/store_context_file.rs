impl<'a> StoreActionContext<'a> {
    async fn execute_file(
        &mut self,
        statement: &ServerFileStatement,
    ) -> Result<(), StoreActionError> {
        match statement {
            ServerFileStatement::Write {
                binding,
                root,
                path,
                data,
                sha256,
            } => {
                let destination = self.file_path(root, path, true)?;
                let bytes = self.bytes_for_reference(data)?;
                let actual_sha256 = Sha256::digest(&bytes)
                    .iter()
                    .map(|value| format!("{value:02x}"))
                    .collect::<String>();
                if let Some(expected) = sha256
                    && self.literal_string(expected)? != actual_sha256
                {
                    return Err(StoreActionError::file_hash_mismatch());
                }
                let parent = destination
                    .parent()
                    .ok_or_else(StoreActionError::invalid_file_path)?;
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|_| StoreActionError::file())?;
                let temporary = destination.with_extension(format!(
                    "dowe-{}-{}",
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos()
                ));
                let mut options = tokio::fs::OpenOptions::new();
                options.write(true).create_new(true);
                let mut file = options
                    .open(&temporary)
                    .await
                    .map_err(|_| StoreActionError::file())?;
                use tokio::io::AsyncWriteExt;
                file.write_all(&bytes)
                    .await
                    .map_err(|_| StoreActionError::file())?;
                file.sync_all()
                    .await
                    .map_err(|_| StoreActionError::file())?;
                drop(file);
                if tokio::fs::rename(&temporary, &destination).await.is_err() {
                    if destination.is_file() {
                        tokio::fs::remove_file(&destination)
                            .await
                            .map_err(|_| StoreActionError::file())?;
                    }
                    tokio::fs::rename(&temporary, &destination)
                        .await
                        .map_err(|_| StoreActionError::file())?;
                }
                self.bindings.insert(
                    binding.clone(),
                    json!({ "written": true, "size": bytes.len(), "sha256": actual_sha256 }),
                );
            }
            ServerFileStatement::Read {
                binding,
                root,
                path,
            } => {
                let source = self.file_path(root, path, false)?;
                let bytes = tokio::fs::read(source)
                    .await
                    .map_err(|error| match error.kind() {
                        std::io::ErrorKind::NotFound => {
                            StoreActionError::not_found("File not found")
                        }
                        _ => StoreActionError::file(),
                    })?;
                self.bytes_results
                    .insert(binding.clone(), Bytes::from(bytes.clone()));
                self.bindings
                    .insert(binding.clone(), bytes_binding_json(bytes.len(), "file"));
            }
            ServerFileStatement::Exists {
                binding,
                root,
                path,
            } => {
                let source = self.file_path(root, path, false)?;
                let exists = source.is_file();
                self.bindings
                    .insert(binding.clone(), json!({ "exists": exists }));
            }
            ServerFileStatement::Delete {
                binding,
                root,
                path,
            } => {
                let source = self.file_path(root, path, false)?;
                let deleted = if source.is_file() {
                    tokio::fs::remove_file(source)
                        .await
                        .map_err(|_| StoreActionError::file())?;
                    true
                } else {
                    false
                };
                self.bindings
                    .insert(binding.clone(), json!({ "deleted": deleted }));
            }
        }
        Ok(())
    }

    fn file_path(
        &self,
        root: &StoreLiteral,
        path: &StoreLiteral,
        create_root: bool,
    ) -> Result<PathBuf, StoreActionError> {
        let root = PathBuf::from(self.literal_string(root)?);
        let root = if root.is_absolute() {
            root
        } else {
            self.root.join(root)
        };
        if create_root {
            fs::create_dir_all(&root).map_err(|_| StoreActionError::file())?;
        }
        if fs::symlink_metadata(&root)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
        {
            return Err(StoreActionError::invalid_file_path());
        }
        let root = root.canonicalize().map_err(|_| StoreActionError::file())?;
        let relative = PathBuf::from(self.literal_string(path)?);
        if !valid_relative_file_path(&relative) {
            return Err(StoreActionError::invalid_file_path());
        }
        reject_symlink_descendants(&root, &relative)?;
        let destination = root.join(relative);
        Ok(destination)
    }
}

fn valid_relative_file_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn reject_symlink_descendants(root: &Path, path: &Path) -> Result<(), StoreActionError> {
    let mut current = root.to_path_buf();
    for component in path.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(StoreActionError::invalid_file_path());
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(StoreActionError::file()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod file_tests {
    use super::valid_relative_file_path;
    use std::path::Path;

    #[test]
    fn file_paths_reject_traversal_and_absolute_values() {
        assert!(valid_relative_file_path(Path::new(
            "account/app/hash.dowebin"
        )));
        assert!(!valid_relative_file_path(Path::new("../secret")));
        assert!(!valid_relative_file_path(Path::new("/etc/passwd")));
        assert!(!valid_relative_file_path(Path::new("")));
    }
}
