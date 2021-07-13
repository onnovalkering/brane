use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use specifications::common::{FunctionExt, Value};

#[repr(u8)]
pub enum ServiceState {
    Created = 1,
    Started = 2,
    Done = 3,
}

#[async_trait]
pub trait VmExecutor {
    async fn call(
        &self,
        call: FunctionExt,
        arguments: HashMap<String, Value>,
        location: Option<String>,
    ) -> Result<Value>;

    async fn wait_until(
        &self,
        service: String,
        state: ServiceState,
    ) -> Result<()>;
}

#[derive(Clone)]
pub struct NoExtExecutor {}

impl Default for NoExtExecutor {
    fn default() -> Self {
        Self {}
    }
}

#[async_trait]
impl VmExecutor for NoExtExecutor {
    async fn call(
        &self,
        _: FunctionExt,
        _: HashMap<String, Value>,
        _: Option<String>,
    ) -> Result<Value> {
        bail!("External function calls not supported.");
    }

    async fn wait_until(
        &self,
        _: String,
        _: ServiceState,
    ) -> Result<()> {
        bail!("External function calls not supported.");
    }
}
