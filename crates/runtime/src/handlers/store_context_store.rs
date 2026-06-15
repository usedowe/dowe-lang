impl<'a> StoreActionContext<'a> {
    async fn execute_store(
        &mut self,
        statement: &ServerStoreStatement,
    ) -> Result<(), StoreActionError> {
        match statement {
            ServerStoreStatement::Handle {
                binding,
                database,
                remote,
            } => {
                let database_name = database.clone();
                let handle = if let Some(remote) = remote {
                    StoreHandle::Remote(self.remote_client(database, remote)?)
                } else {
                    init_database(self.root, database).map_err(StoreActionError::from_store)?;
                    let database =
                        open_database(self.root, database).map_err(StoreActionError::from_store)?;
                    StoreHandle::Local(database)
                };
                self.handles.insert(binding.clone(), handle);
                self.handle_databases.insert(binding.clone(), database_name);
            }
            ServerStoreStatement::List {
                binding,
                handle,
                table,
            } => {
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => {
                        let records = database
                            .records(table)
                            .map_err(StoreActionError::from_store)?;
                        Value::Array(records.iter().map(record_json).collect())
                    }
                    StoreHandle::Remote(client) => client
                        .list(table)
                        .await
                        .map_err(StoreActionError::from_store)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerStoreStatement::Read {
                binding,
                handle,
                table,
                filter,
                required,
            } => {
                let expected = self.filter_value(filter)?;
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => {
                        let record = database
                            .records(table)
                            .map_err(StoreActionError::from_store)?
                            .into_iter()
                            .find(|record| record_matches(record, &filter.field, &expected));
                        if record.is_none() && *required {
                            return Err(StoreActionError::not_found("Record not found"));
                        }
                        record.as_ref().map(record_json).unwrap_or(Value::Null)
                    }
                    StoreHandle::Remote(client) => client
                        .read(table, &filter.field, expected.to_json(), *required)
                        .await
                        .map_err(StoreActionError::from_store)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerStoreStatement::Insert {
                binding,
                handle,
                table,
                value,
                required,
            } => {
                let record = self.literal_record(value)?;
                validate_required_fields(&record, required)?;
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => {
                        let inserted = database
                            .insert(table, record)
                            .map_err(StoreActionError::from_store)?;
                        record_json(&inserted)
                    }
                    StoreHandle::Remote(client) => client
                        .insert(table, record_json(&record))
                        .await
                        .map_err(StoreActionError::from_store)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerStoreStatement::Update {
                binding,
                handle,
                table,
                filter,
                value,
                required,
                matches,
            } => {
                self.validate_matches(matches)?;
                let expected = self.filter_value(filter)?;
                let patch = self.literal_record(value)?;
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => {
                        let changed = database
                            .update(table, &filter.field, &expected, patch)
                            .map_err(StoreActionError::from_store)?;
                        if changed == 0 && *required {
                            return Err(StoreActionError::not_found("Record not found"));
                        }
                        changed_json(changed)
                    }
                    StoreHandle::Remote(client) => client
                        .update(
                            table,
                            &filter.field,
                            expected.to_json(),
                            record_json(&patch),
                            *required,
                        )
                        .await
                        .map_err(StoreActionError::from_store)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerStoreStatement::Delete {
                binding,
                handle,
                table,
                filter,
                required,
            } => {
                let expected = self.filter_value(filter)?;
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => {
                        let changed = database
                            .delete(table, &filter.field, &expected)
                            .map_err(StoreActionError::from_store)?;
                        if changed == 0 && *required {
                            return Err(StoreActionError::not_found("Record not found"));
                        }
                        changed_json(changed)
                    }
                    StoreHandle::Remote(client) => client
                        .delete(table, &filter.field, expected.to_json(), *required)
                        .await
                        .map_err(StoreActionError::from_store)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerStoreStatement::Query {
                binding,
                handle,
                sql,
            } => {
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => database
                        .query_json(sql)
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::Remote(client) => client
                        .query(sql)
                        .await
                        .map_err(StoreActionError::from_store)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerStoreStatement::Transaction {
                binding,
                handle,
                operations,
                return_binding,
            } => {
                if matches!(self.handle(handle)?, StoreHandle::Remote(_)) {
                    return Err(StoreActionError::store());
                }
                let database_name = self
                    .handle_databases
                    .get(handle)
                    .cloned()
                    .ok_or_else(StoreActionError::store)?;
                let transaction = StoreTransactionEndpoint {
                    database: database_name,
                    operations: operations.clone(),
                    return_binding: return_binding.clone(),
                };
                let value = execute_store_transaction(self.root, &transaction)
                    .map_err(|_| StoreActionError::store())?;
                self.bindings.insert(binding.clone(), value);
            }
        }
        Ok(())
    }
}
