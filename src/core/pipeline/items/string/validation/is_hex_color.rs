use async_trait::async_trait;
use regex::Regex;
use crate::core::pipeline::item::Item;
use crate::core::pipeline::ctx::Ctx;
use crate::core::result::Result;

#[derive(Debug, Clone)]
pub struct IsHexColorModifier {
    regex: Regex
}

impl IsHexColorModifier {
    pub fn new() -> Self {
        return IsHexColorModifier {
            regex: Regex::new(r"^[A-Fa-f0-9]{6}$").unwrap()
        };
    }
}

#[async_trait]
impl Item for IsHexColorModifier {
    async fn call<'a>(&self, ctx: Ctx<'a>) -> Result<Ctx<'a>> {
        match ctx.value.as_str() {
            Some(s) => {
                if self.regex.is_match(s) {
                    Ok(ctx)
                } else {
                    Err(ctx.with_invalid("String is not hex color."))
                }
            }
            None => {
                Err(ctx.internal_server_error("isHexColor: value is not string"))
            }
        }
    }
}