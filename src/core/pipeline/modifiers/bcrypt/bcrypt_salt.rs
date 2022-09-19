use async_trait::async_trait;
use bcrypt::{DEFAULT_COST, hash};
use crate::core::pipeline::modifier::Modifier;
use crate::core::value::Value;
use crate::core::object::Object;
use crate::core::pipeline::context::Context;
use crate::core::pipeline::context::Validity::Invalid;

#[derive(Debug, Copy, Clone)]
pub struct BcryptSaltModifier {}

impl BcryptSaltModifier {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Modifier for BcryptSaltModifier {

    fn name(&self) -> &'static str {
        "bcryptSalt"
    }

    async fn call(&self, context: Context) -> Context {
        match context.value.as_str() {
            Some(s) => {
                context.alter_value(Value::String(hash(s.as_str(), DEFAULT_COST).unwrap()))
            }
            None => {
                context.alter_validity(Invalid("Value is not string.".to_owned()))
            }
        }
    }
}