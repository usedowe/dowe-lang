impl DoweQueue {
    pub fn open(project_root: &Path, name: &str) -> QueueResult<Self> {
        open_namespace(project_root, name)
    }

    pub fn init(project_root: &Path, name: &str) -> QueueResult<()> {
        init_namespace(project_root, name)
    }

    pub fn name(&self) -> String {
        self.shared
            .state
            .lock()
            .map(|state| state.name.clone())
            .unwrap_or_default()
    }

    pub fn declare(&self, queue: &str) -> QueueResult<DeclareReport> {
        validate_queue_name(queue)?;
        let mut state = self.lock_state()?;
        let created = if state.queues.contains_key(queue) {
            false
        } else {
            state.queues.insert(
                queue.to_string(),
                PersistedQueue {
                    bindings: BTreeSet::new(),
                    ready: VecDeque::new(),
                    in_flight: BTreeMap::new(),
                },
            );
            persist(&self.shared.path, &state)?;
            true
        };
        Ok(DeclareReport {
            queue: queue.to_string(),
            created: Some(created),
        })
    }

    pub fn bind(&self, queue: &str, pattern: &str) -> QueueResult<BindReport> {
        validate_queue_name(queue)?;
        validate_pattern(pattern)?;
        let mut state = self.lock_state()?;
        let queue_state = state
            .queues
            .get_mut(queue)
            .ok_or_else(|| QueueError::QueueNotFound("Queue does not exist".to_string()))?;
        let created = queue_state.bindings.insert(pattern.to_string());
        if created {
            persist(&self.shared.path, &state)?;
        }
        Ok(BindReport {
            queue: queue.to_string(),
            pattern: pattern.to_string(),
            created: Some(created),
        })
    }

    pub fn publish(&self, topic: &str, value: Value) -> QueueResult<PublishReport> {
        validate_topic(topic)?;
        let message = QueueMessage {
            id: generate_ulid(),
            topic: topic.to_string(),
            value,
            published_at: timestamp(),
            redelivered: false,
        };
        let mut state = self.lock_state()?;
        let mut destinations = Vec::new();
        for (queue, queue_state) in &mut state.queues {
            if queue_state
                .bindings
                .iter()
                .any(|pattern| topic_matches(pattern, topic))
            {
                queue_state.ready.push_back(message.clone());
                destinations.push(queue.clone());
            }
        }
        persist(&self.shared.path, &state)?;
        drop(state);
        if !destinations.is_empty() {
            self.shared.notify.notify_waiters();
        }
        Ok(PublishReport {
            id: message.id,
            destinations: Some(destinations),
            confirmed: true,
        })
    }

    pub fn publish_direct(&self, queue: &str, value: Value) -> QueueResult<DirectPublishReport> {
        validate_queue_name(queue)?;
        let message = QueueMessage {
            id: generate_ulid(),
            topic: queue.to_string(),
            value,
            published_at: timestamp(),
            redelivered: false,
        };
        let mut state = self.lock_state()?;
        let queue_state = state
            .queues
            .get_mut(queue)
            .ok_or_else(|| QueueError::QueueNotFound("Queue does not exist".to_string()))?;
        queue_state.ready.push_back(message.clone());
        persist(&self.shared.path, &state)?;
        drop(state);
        self.shared.notify.notify_waiters();
        Ok(DirectPublishReport {
            id: message.id,
            confirmed: true,
        })
    }

    pub fn inspect(&self) -> QueueResult<QueueInspection> {
        let state = self.lock_state()?;
        let queues = state
            .queues
            .iter()
            .map(|(name, queue)| QueueInspectionEntry {
                queue: name.clone(),
                bindings: queue.bindings.iter().cloned().collect(),
                ready: queue.ready.len(),
                in_flight: queue.in_flight.len(),
            })
            .collect();
        Ok(QueueInspection {
            name: state.name.clone(),
            queues: Some(queues),
        })
    }

    pub fn inspect_messages(&self, queue: &str, limit: usize) -> QueueResult<Vec<QueueMessage>> {
        validate_queue_name(queue)?;
        let state = self.lock_state()?;
        let queue_state = state
            .queues
            .get(queue)
            .ok_or_else(|| QueueError::QueueNotFound("Queue does not exist".to_string()))?;
        let limit = limit.clamp(1, 100);
        let mut messages = queue_state
            .ready
            .iter()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();
        if messages.len() < limit {
            messages.extend(
                queue_state
                    .in_flight
                    .values()
                    .take(limit.saturating_sub(messages.len()))
                    .map(|delivery| delivery.message.clone()),
            );
        }
        Ok(messages)
    }

    pub fn purge(&self, queue: &str) -> QueueResult<PurgeReport> {
        validate_queue_name(queue)?;
        let mut state = self.lock_state()?;
        let queue_state = state
            .queues
            .get_mut(queue)
            .ok_or_else(|| QueueError::QueueNotFound("Queue does not exist".to_string()))?;
        let removed = queue_state.ready.len();
        queue_state.ready.clear();
        persist(&self.shared.path, &state)?;
        Ok(PurgeReport {
            queue: queue.to_string(),
            removed,
        })
    }

    pub fn subscribe(&self, queue: &str, consumer: &str) -> QueueResult<DoweSubscription> {
        validate_queue_name(queue)?;
        validate_consumer_name(consumer)?;
        let state = self.lock_state()?;
        if !state.queues.contains_key(queue) {
            return Err(QueueError::QueueNotFound(
                "Queue does not exist".to_string(),
            ));
        }
        drop(state);
        Ok(DoweSubscription {
            engine: self.clone(),
            queue: queue.to_string(),
            session: generate_ulid(),
            closed: false,
        })
    }

    fn take(&self, queue: &str, session: &str) -> QueueResult<Option<LocalDelivery>> {
        let mut state = self.lock_state()?;
        let queue_state = state
            .queues
            .get_mut(queue)
            .ok_or_else(|| QueueError::QueueNotFound("Queue does not exist".to_string()))?;
        let Some(message) = queue_state.ready.pop_front() else {
            return Ok(None);
        };
        let receipt = generate_ulid();
        queue_state.in_flight.insert(
            receipt.clone(),
            InFlight {
                session: session.to_string(),
                message: message.clone(),
            },
        );
        persist(&self.shared.path, &state)?;
        Ok(Some(LocalDelivery { message, receipt }))
    }

    fn settle(&self, queue: &str, session: &str, receipt: &str, requeue: bool) -> QueueResult<()> {
        let mut state = self.lock_state()?;
        let queue_state = state
            .queues
            .get_mut(queue)
            .ok_or_else(|| QueueError::QueueNotFound("Queue does not exist".to_string()))?;
        let in_flight = queue_state.in_flight.remove(receipt).ok_or_else(|| {
            QueueError::InvalidReceipt("Queue delivery receipt is invalid or expired".to_string())
        })?;
        if in_flight.session != session {
            queue_state.in_flight.insert(receipt.to_string(), in_flight);
            return Err(QueueError::InvalidReceipt(
                "Queue delivery receipt does not belong to this subscription".to_string(),
            ));
        }
        if requeue {
            let mut message = in_flight.message;
            message.redelivered = true;
            queue_state.ready.push_front(message);
        }
        persist(&self.shared.path, &state)?;
        drop(state);
        if requeue {
            self.shared.notify.notify_waiters();
        }
        Ok(())
    }

    fn requeue_session(&self, queue: &str, session: &str) -> QueueResult<()> {
        let mut state = self.lock_state()?;
        let queue_state = state
            .queues
            .get_mut(queue)
            .ok_or_else(|| QueueError::QueueNotFound("Queue does not exist".to_string()))?;
        let receipts = queue_state
            .in_flight
            .iter()
            .filter(|(_, delivery)| delivery.session == session)
            .map(|(receipt, _)| receipt.clone())
            .collect::<Vec<_>>();
        if receipts.is_empty() {
            return Ok(());
        }
        let mut messages = Vec::with_capacity(receipts.len());
        for receipt in receipts {
            if let Some(mut in_flight) = queue_state.in_flight.remove(&receipt) {
                in_flight.message.redelivered = true;
                messages.push(in_flight.message);
            }
        }
        for message in messages.into_iter().rev() {
            queue_state.ready.push_front(message);
        }
        persist(&self.shared.path, &state)?;
        drop(state);
        self.shared.notify.notify_waiters();
        Ok(())
    }

    fn lock_state(&self) -> QueueResult<std::sync::MutexGuard<'_, QueueState>> {
        self.shared
            .state
            .lock()
            .map_err(|_| QueueError::DurabilityError("Queue engine lock failed".to_string()))
    }
}

