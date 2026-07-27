impl<'a> StoreActionContext<'a> {
    async fn execute_vector(
        &mut self,
        statement: &ServerVectorStatement,
    ) -> Result<(), StoreActionError> {
        match statement {
            ServerVectorStatement::Handle { connection } => {
                let name = self.vector_connection_value(&connection.name)?;
                let local = matches!(self.cache_mode, CacheRuntimeMode::Local)
                    || self
                        .vector_connection_value(&connection.host)?
                        .eq_ignore_ascii_case("local");
                let handle = if local {
                    VectorHandle::Local(
                        open_vector_database(self.root, &name, true)
                            .map_err(StoreActionError::from_vector)?,
                    )
                } else {
                    VectorHandle::Remote(self.vector_remote_client(connection)?)
                };
                self.vector_handles
                    .insert(connection.binding.clone(), handle);
            }
            ServerVectorStatement::Upsert {
                binding,
                handle,
                id,
                vector,
                metadata,
            } => {
                let id = self.vector_id(id)?;
                let vector = self.vector_value(vector)?;
                let metadata = metadata
                    .as_ref()
                    .map(|value| self.vector_metadata(value))
                    .transpose()?
                    .unwrap_or_else(|| Value::Object(Map::new()));
                let output = match self.vector_handle(handle)? {
                    VectorHandle::Local(database) => serde_json::to_value(
                        database
                            .upsert(&id, vector, metadata)
                            .map_err(StoreActionError::from_vector)?,
                    )
                    .map_err(|_| StoreActionError::vector())?,
                    VectorHandle::Remote(client) => client
                        .upsert(&id, vector, metadata)
                        .await
                        .map_err(StoreActionError::from_vector)?,
                };
                self.bindings.insert(binding.clone(), output);
            }
            ServerVectorStatement::Search {
                binding,
                handle,
                vector,
                limit,
                min_score,
                filter,
            } => {
                let vector = self.vector_value(vector)?;
                let filter = filter
                    .as_ref()
                    .map(|value| self.vector_metadata(value))
                    .transpose()?;
                let min_score = min_score
                    .parse::<f32>()
                    .map_err(|_| StoreActionError::vector())?;
                let output = match self.vector_handle(handle)? {
                    VectorHandle::Local(database) => serde_json::to_value(
                        database
                            .search(&vector, *limit, min_score, filter.as_ref())
                            .map_err(StoreActionError::from_vector)?,
                    )
                    .map_err(|_| StoreActionError::vector())?,
                    VectorHandle::Remote(client) => client
                        .search(vector, *limit, min_score, filter)
                        .await
                        .map_err(StoreActionError::from_vector)?,
                };
                self.bindings.insert(binding.clone(), output);
            }
            ServerVectorStatement::Read {
                binding,
                handle,
                id,
                required,
            } => {
                let id = self.vector_id(id)?;
                let output = match self.vector_handle(handle)? {
                    VectorHandle::Local(database) => {
                        let value = database
                            .read(&id)
                            .map_err(StoreActionError::from_vector)?;
                        if value.is_none() && *required {
                            return Err(StoreActionError::not_found(
                                "Embedding not found",
                            ));
                        }
                        value
                            .map(serde_json::to_value)
                            .transpose()
                            .map_err(|_| StoreActionError::vector())?
                            .unwrap_or(Value::Null)
                    }
                    VectorHandle::Remote(client) => client
                        .read(&id, *required)
                        .await
                        .map_err(StoreActionError::from_vector)?,
                };
                self.bindings.insert(binding.clone(), output);
            }
            ServerVectorStatement::Delete {
                binding,
                handle,
                id,
            } => {
                let id = self.vector_id(id)?;
                let output = match self.vector_handle(handle)? {
                    VectorHandle::Local(database) => {
                        let mut value = Map::new();
                        value.insert(
                            "deleted".to_string(),
                            Value::Bool(
                                database
                                    .delete(&id)
                                    .map_err(StoreActionError::from_vector)?,
                            ),
                        );
                        Value::Object(value)
                    }
                    VectorHandle::Remote(client) => client
                        .delete(&id)
                        .await
                        .map_err(StoreActionError::from_vector)?,
                };
                self.bindings.insert(binding.clone(), output);
            }
            ServerVectorStatement::List {
                binding,
                handle,
                limit,
                filter,
            } => {
                let filter = filter
                    .as_ref()
                    .map(|value| self.vector_metadata(value))
                    .transpose()?;
                let output = match self.vector_handle(handle)? {
                    VectorHandle::Local(database) => serde_json::to_value(
                        database
                            .list(*limit, filter.as_ref())
                            .map_err(StoreActionError::from_vector)?,
                    )
                    .map_err(|_| StoreActionError::vector())?,
                    VectorHandle::Remote(client) => client
                        .list(*limit, filter)
                        .await
                        .map_err(StoreActionError::from_vector)?,
                };
                self.bindings.insert(binding.clone(), output);
            }
        }
        Ok(())
    }

    fn vector_id(&self, value: &StoreLiteral) -> Result<String, StoreActionError> {
        self.evaluate(value)?
            .into_json()
            .and_then(|value| value.as_str().map(str::to_string))
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                StoreActionError::invalid_body(
                    "Vector embedding ID must be a non-empty string",
                )
            })
    }

    fn vector_value(&self, value: &StoreLiteral) -> Result<Vec<f32>, StoreActionError> {
        let values = self
            .evaluate(value)?
            .into_json()
            .and_then(|value| value.as_array().cloned())
            .ok_or_else(|| {
                StoreActionError::invalid_body(
                    "Vector value must be an array of numbers",
                )
            })?;
        values
            .into_iter()
            .map(|value| {
                value
                    .as_f64()
                    .map(|value| value as f32)
                    .filter(|value| value.is_finite())
                    .ok_or_else(|| {
                        StoreActionError::invalid_body(
                            "Vector dimensions must be finite numbers",
                        )
                    })
            })
            .collect()
    }

    fn vector_metadata(&self, value: &StoreLiteral) -> Result<Value, StoreActionError> {
        self.evaluate(value)?
            .into_json()
            .filter(Value::is_object)
            .ok_or_else(|| {
                StoreActionError::invalid_body(
                    "Vector metadata and where must be objects",
                )
            })
    }
}
