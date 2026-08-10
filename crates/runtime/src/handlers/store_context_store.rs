impl<'a> StoreActionContext<'a> {
    async fn execute_store(
        &mut self,
        statement: &ServerStoreStatement,
    ) -> Result<(), StoreActionError> {
        match statement {
            ServerStoreStatement::Handle { connection } => {
                let database_name = connection.database.clone();
                let handle = if self.project.local_databases {
                    init_database(self.root, &connection.database)
                        .map_err(StoreActionError::from_store)?;
                    StoreHandle::Local(
                        open_database(self.root, &connection.database)
                            .map_err(StoreActionError::from_store)?,
                    )
                } else {
                    self.configured_database_client(connection)?
                };
                self.handles.insert(connection.binding.clone(), handle);
                self.handle_databases
                    .insert(connection.binding.clone(), database_name);
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
                    StoreHandle::Dowe(client) => client
                        .list(table)
                        .await
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::D1(client) => client
                        .list(table)
                        .await
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::Postgres(client) => client
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
                let filters = self.filter_values(filter)?;
                let json_filters = json_filters(&filters);
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => {
                        let record = database
                            .records(table)
                            .map_err(StoreActionError::from_store)?
                            .into_iter()
                            .find(|record| record_matches_all(record, &filters));
                        if record.is_none() && *required {
                            return Err(StoreActionError::not_found("Record not found"));
                        }
                        record.as_ref().map(record_json).unwrap_or(Value::Null)
                    }
                    StoreHandle::Dowe(client) => client
                        .read(table, json_filters, *required)
                        .await
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::D1(client) => client
                        .read(table, &json_filters, *required)
                        .await
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::Postgres(client) => client
                        .read(table, &json_filters, *required)
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
                let json = record_json(&record);
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => record_json(
                        &database
                            .insert(table, record)
                            .map_err(StoreActionError::from_store)?,
                    ),
                    StoreHandle::Dowe(client) => client
                        .insert(table, json)
                        .await
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::D1(client) => client
                        .insert(table, json)
                        .await
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::Postgres(client) => client
                        .insert(table, json)
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
                let filters = self.filter_values(filter)?;
                let json_filters = json_filters(&filters);
                let patch = self.literal_record(value)?;
                let json_patch = record_json(&patch);
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => {
                        let records = database
                            .records(table)
                            .map_err(StoreActionError::from_store)?;
                        let mut changed = 0usize;
                        for record in records
                            .into_iter()
                            .filter(|record| record_matches_all(record, &filters))
                        {
                            let Some(id) = record.get("id") else {
                                continue;
                            };
                            changed += database
                                .update(table, "id", id, patch.clone())
                                .map_err(StoreActionError::from_store)?;
                        }
                        if changed == 0 && *required {
                            return Err(StoreActionError::not_found("Record not found"));
                        }
                        changed_json(changed)
                    }
                    StoreHandle::Dowe(client) => client
                        .update(table, json_filters, json_patch, *required)
                        .await
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::D1(client) => client
                        .update(table, &json_filters, json_patch, *required)
                        .await
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::Postgres(client) => client
                        .update(table, &json_filters, json_patch, *required)
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
                let filters = self.filter_values(filter)?;
                let json_filters = json_filters(&filters);
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => {
                        let records = database
                            .records(table)
                            .map_err(StoreActionError::from_store)?;
                        let mut changed = 0usize;
                        for record in records
                            .into_iter()
                            .filter(|record| record_matches_all(record, &filters))
                        {
                            let Some(id) = record.get("id") else {
                                continue;
                            };
                            changed += database
                                .delete(table, "id", id)
                                .map_err(StoreActionError::from_store)?;
                        }
                        if changed == 0 && *required {
                            return Err(StoreActionError::not_found("Record not found"));
                        }
                        changed_json(changed)
                    }
                    StoreHandle::Dowe(client) => client
                        .delete(table, json_filters, *required)
                        .await
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::D1(client) => client
                        .delete(table, &json_filters, *required)
                        .await
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::Postgres(client) => client
                        .delete(table, &json_filters, *required)
                        .await
                        .map_err(StoreActionError::from_store)?,
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerStoreStatement::Query {
                binding,
                handle,
                sql,
                params,
            } => {
                let params = self
                    .evaluate(&StoreLiteral::Array(params.clone()))?
                    .into_json()
                    .and_then(|value| value.as_array().cloned())
                    .ok_or_else(StoreActionError::store)?;
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => database
                        .query_json(
                            &bind_query_params(sql, &params)
                                .map_err(StoreActionError::from_store)?,
                        )
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::Dowe(client) => client
                        .query_with_params(sql, &params)
                        .await
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::D1(client) => client
                        .query_with_params(sql, &params)
                        .await
                        .map_err(StoreActionError::from_store)?,
                    StoreHandle::Postgres(client) => client
                        .query_with_params(sql, &params)
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
                let transaction = StoreTransactionEndpoint {
                    connection: self
                        .project
                        .databases
                        .iter()
                        .find(|binding| binding.binding == *handle)
                        .map(|binding| binding.connection.clone())
                        .ok_or_else(StoreActionError::store)?,
                    operations: operations.clone(),
                    return_binding: return_binding.clone(),
                };
                let value = match self.handle(handle)? {
                    StoreHandle::Local(database) => {
                        execute_local_store_transaction(database, &transaction)
                            .map_err(StoreActionError::from_store)?
                    }
                    StoreHandle::Dowe(client) => {
                        let committed = client
                            .transaction(&transaction_insert_requests(operations))
                            .await
                            .map_err(StoreActionError::from_store)?;
                        transaction_result(committed, &transaction)
                            .map_err(StoreActionError::from_store)?
                    }
                    StoreHandle::D1(_) | StoreHandle::Postgres(_) => {
                        return Err(StoreActionError::store());
                    }
                };
                self.bindings.insert(binding.clone(), value);
            }
        }
        Ok(())
    }
}

fn json_filters(filters: &[(String, StoreValue)]) -> Vec<(String, Value)> {
    filters
        .iter()
        .map(|(field, value)| (field.clone(), value.to_json()))
        .collect()
}
