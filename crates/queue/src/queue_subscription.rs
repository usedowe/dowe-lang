impl DoweSubscription {
    pub async fn next(&mut self) -> QueueResult<Option<QueueDelivery>> {
        let Some(local_delivery) = self.next_raw().await? else {
            return Ok(None);
        };
        Ok(Some(delivery(
            local_delivery.message,
            LocalReceipt {
                engine: self.engine.clone(),
                queue: self.queue.clone(),
                session: self.session.clone(),
                receipt: local_delivery.receipt,
                resolved: false,
            },
        )))
    }

    pub async fn close(&mut self) -> QueueResult<()> {
        self.close_sync()
    }

    pub(crate) async fn next_frame(&mut self) -> QueueResult<Option<QueueDeliveryFrame>> {
        Ok(self.next_raw().await?.map(|delivery| QueueDeliveryFrame {
            message: delivery.message,
            receipt: delivery.receipt,
        }))
    }

    pub(crate) fn ack_token(&self, receipt: &str) -> QueueResult<()> {
        self.engine
            .settle(&self.queue, &self.session, receipt, false)
    }

    pub(crate) fn nack_token(&self, receipt: &str, requeue: bool) -> QueueResult<()> {
        self.engine
            .settle(&self.queue, &self.session, receipt, requeue)
    }

    async fn next_raw(&mut self) -> QueueResult<Option<LocalDelivery>> {
        if self.closed {
            return Ok(None);
        }
        loop {
            let notified = self.engine.shared.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if let Some(delivery) = self.engine.take(&self.queue, &self.session)? {
                return Ok(Some(delivery));
            }
            notified.await;
            if self.closed {
                return Ok(None);
            }
        }
    }

    fn close_sync(&mut self) -> QueueResult<()> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        self.engine.requeue_session(&self.queue, &self.session)
    }
}

