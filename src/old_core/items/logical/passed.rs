use async_trait::async_trait;
use crate::core::item::Item;
use crate::core::pipeline::Pipeline;
use crate::core::pipeline::ctx::PipelineCtx;
use teo_teon::value::Value;
use crate::core::result::Result;

#[derive(Debug, Clone)]
pub struct PassedItem {
    pipeline: Pipeline
}

impl PassedItem {
    pub fn new(pipeline: Pipeline) -> Self {
        return PassedItem {
            pipeline
        };
    }
}

#[async_trait]
impl Item for PassedItem {
    async fn call<'a>(&self, ctx: PipelineCtx<'a>) -> Result<PipelineCtx<'a>> {
        match self.pipeline.process(ctx.clone()).await {
            Ok(_) => Ok(ctx.with_value(Value::Bool(true))),
            Err(error) => if error.is_server_error() {
                Err(error)
            } else {
                Ok(ctx.with_value(Value::Bool(false)))
            }
        }
    }
}