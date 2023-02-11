use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::sync::Arc;
use async_trait::async_trait;
use futures_util::future::BoxFuture;
use crate::core::pipeline::item::Item;
use crate::core::pipeline::ctx::Ctx;
use crate::core::teon::Value;
use crate::core::result::Result;

pub trait PerformArgument<T: From<Value> + Send + Sync>: Send + Sync {
    fn call(&self, args: T) -> BoxFuture<'static, ()>;
}

impl<T, F, Fut> PerformArgument<T> for F where
T: From<Value> + Send + Sync,
F: Fn(T) -> Fut + Sync + Send,
Fut: Future<Output = ()> + Send + Sync + 'static {
    fn call(&self, args: T) -> BoxFuture<'static, ()> {
        Box::pin(self(args))
    }
}

#[derive(Clone)]
pub struct PerformModifier<T> {
    callback: Arc<dyn PerformArgument<T>>
}

impl<T> Debug for PerformModifier<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = f.debug_struct("PerformModifier");
        result.finish()
    }
}

impl<T> PerformModifier<T> {
    pub fn new<F>(f: F) -> PerformModifier<T> where
        T: From<Value> + Send + Sync,
        F: PerformArgument<T> + 'static {
        return PerformModifier {
            callback: Arc::new(f)
        }
    }
}

#[async_trait]
impl<T: From<Value> + Send + Sync> Item for PerformModifier<T> {
    async fn call<'a>(&self, ctx: Ctx<'a>) -> Result<Ctx<'a>> {
        let cb = self.callback.clone();
        cb.call((&ctx).value.clone().into()).await;
        Ok(ctx)
    }
}

unsafe impl<T> Send for PerformModifier<T> {}
unsafe impl<T> Sync for PerformModifier<T> {}