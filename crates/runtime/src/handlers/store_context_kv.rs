impl<'a> StoreActionContext<'a> {
    async fn execute_kv(&mut self, statement: &ServerKvStatement) -> Result<(), StoreActionError> {
        match statement {
            ServerKvStatement::Handle {
                binding,
                database,
                persist,
                remote,
            } => {
                let handle = if let Some(remote) = remote {
                    KvHandle::Remote(self.kv_remote_client(database, *persist, remote)?)
                } else {
                    KvHandle::Local(
                        open_kv_database(self.root, database, *persist)
                            .map_err(StoreActionError::from_kv)?,
                    )
                };
                self.kv_handles.insert(binding.clone(), handle);
            }
            ServerKvStatement::Get {
                binding,
                handle,
                key,
                required,
            } => {
                let value = match self.kv_handle(handle)? {
                    KvHandle::Local(database) => {
                        let value = database.get(key).map_err(StoreActionError::from_kv)?;
                        if value.is_none() && *required {
                            return Err(StoreActionError::not_found("KV key not found"));
                        }
                        value.unwrap_or(Value::Null)
                    }
                    KvHandle::Remote(client) => client
                        .get(key, *required)
                        .await
                        .map_err(StoreActionError::from_kv)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerKvStatement::Set {
                binding,
                handle,
                key,
                value,
            } => {
                let value = self.evaluate(value)?.into_json().unwrap_or(Value::Null);
                let output = match self.kv_handle(handle)? {
                    KvHandle::Local(database) => {
                        database
                            .set(key, value)
                            .map_err(StoreActionError::from_kv)?;
                        kv_set_json(key)
                    }
                    KvHandle::Remote(client) => client
                        .set(key, value)
                        .await
                        .map_err(StoreActionError::from_kv)?,
                };
                self.bindings.insert(binding.clone(), output);
            }
            ServerKvStatement::Delete {
                binding,
                handle,
                key,
            } => {
                let output = match self.kv_handle(handle)? {
                    KvHandle::Local(database) => {
                        kv_delete_json(database.delete(key).map_err(StoreActionError::from_kv)?)
                    }
                    KvHandle::Remote(client) => client
                        .delete(key)
                        .await
                        .map_err(StoreActionError::from_kv)?,
                };
                self.bindings.insert(binding.clone(), output);
            }
            ServerKvStatement::Keys {
                binding,
                handle,
                prefix,
            } => {
                let output = match self.kv_handle(handle)? {
                    KvHandle::Local(database) => Value::Array(
                        database
                            .keys(prefix.as_deref())
                            .map_err(StoreActionError::from_kv)?
                            .into_iter()
                            .map(Value::String)
                            .collect(),
                    ),
                    KvHandle::Remote(client) => client
                        .keys(prefix.as_deref())
                        .await
                        .map_err(StoreActionError::from_kv)?,
                };
                self.bindings.insert(binding.clone(), output);
            }
            ServerKvStatement::Clear { binding, handle } => {
                let output = match self.kv_handle(handle)? {
                    KvHandle::Local(database) => {
                        kv_clear_json(database.clear().map_err(StoreActionError::from_kv)?)
                    }
                    KvHandle::Remote(client) => {
                        client.clear().await.map_err(StoreActionError::from_kv)?
                    }
                };
                self.bindings.insert(binding.clone(), output);
            }
        }
        Ok(())
    }
}
