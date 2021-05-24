use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use specifications::common::{FunctionExt, Value};

#[async_trait]
pub trait VmExecutor {
    async fn call(
        &self,
        call: FunctionExt,
        arguments: HashMap<String, Value>,
        location: Option<String>,
    ) -> Result<Value>;
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
}
