use std::sync::Arc;

use crate::domain::daemon::inbound::{PausePort, QueryPort, QueryResponse, ResumePort, SkipPort};
use crate::domain::daemon::worker::{QueryResponse as WorkerQueryResponse, WorkerHandle};

#[derive(Debug)]
pub struct PauseService {
    worker: Arc<WorkerHandle>,
}

impl PauseService {
    pub fn new(worker: Arc<WorkerHandle>) -> Self {
        Self { worker }
    }
}

#[async_trait::async_trait]
impl PausePort for PauseService {
    async fn pause(&self) {
        self.worker.pause().await
    }
}

#[derive(Debug)]
pub struct ResumeService {
    worker: Arc<WorkerHandle>,
}

impl ResumeService {
    pub fn new(worker: Arc<WorkerHandle>) -> Self {
        Self { worker }
    }
}

#[async_trait::async_trait]
impl ResumePort for ResumeService {
    async fn resume(&self) {
        self.worker.resume().await
    }
}

#[derive(Debug)]
pub struct QueryService {
    worker: Arc<WorkerHandle>,
}

impl QueryService {
    pub fn new(worker: Arc<WorkerHandle>) -> Self {
        Self { worker }
    }
}

#[async_trait::async_trait]
impl QueryPort for QueryService {
    async fn query(&self) -> QueryResponse {
        let WorkerQueryResponse {
            current,
            total,
            past,
            stage,
        } = self.worker.query().await;
        QueryResponse {
            current,
            stage: stage.to_string(),
            total,
            remaining: total - past,
            past,
        }
    }
}

#[derive(Debug)]
pub struct SkipService {
    worker: Arc<WorkerHandle>,
}

impl SkipService {
    pub fn new(worker: Arc<WorkerHandle>) -> Self {
        Self { worker }
    }
}

#[async_trait::async_trait]
impl SkipPort for SkipService {
    async fn skip(&self) {
        self.worker.skip().await
    }
}
