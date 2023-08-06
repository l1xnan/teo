use std::fmt::{Display, Formatter};
use crate::parser::ast::identifier::ASTIdentifier;
use crate::parser::ast::span::Span;

#[derive(Debug)]
pub(crate) struct TypeWithGenerics {
    pub(crate) name: ASTIdentifier,
    pub(crate) args: Vec<TypeWithGenerics>,
    pub(crate) span: Span,
}

impl Display for TypeWithGenerics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name.name)?;
        if self.args.len() > 0 {
            f.write_str("<")?;
        }
        for (index, arg) in self.args.iter().enumerate() {
            Display::fmt(arg, f)?;
            if index != self.args.len() - 1 {
                f.write_str(", ")?;
            }
        }
        if self.args.len() > 0 {
            f.write_str(">")?;
        }
        Ok(())
    }
}