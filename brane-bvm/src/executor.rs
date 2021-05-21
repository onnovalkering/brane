use crate::{objects::FunctionExt, values::Value};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait VmExecutor {
    async fn call(
        &self,
        call: FunctionExt,
        arguments: Vec<Value>,
    ) -> Result<Value>;
}

#[derive(Clone)]
pub struct NoOpExecutor {}

impl Default for NoOpExecutor {
    fn default() -> Self {
        Self {}
    }
}

#[async_trait]
impl VmExecutor for NoOpExecutor {
    async fn call(
        &self,
        _: FunctionExt,
        _: Vec<Value>,
    ) -> Result<Value> {
        bail!("External function calls not supported.");
    }
}
