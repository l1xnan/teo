use crate::parser::ast::comment_block::CommentBlock;
use crate::parser::ast::decorator::ASTDecorator;
use crate::parser::ast::identifier::ASTIdentifier;
use crate::parser::ast::r#type::ASTFieldType;
use crate::parser::ast::span::Span;

#[derive(Debug, Copy, Clone)]
pub(crate) enum ASTFieldClass {
    Unresolved,
    Field,
    DroppedField,
    Relation,
    Property,
}

impl ASTFieldClass {
    pub(crate) fn is_relation(&self) -> bool {
        match self {
            ASTFieldClass::Relation => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ASTField {
    pub(crate) source_id: usize,
    pub(crate) comment_block: Option<CommentBlock>,
    pub(crate) identifier: ASTIdentifier,
    pub(crate) r#type: ASTFieldType,
    pub(crate) decorators: Vec<ASTDecorator>,
    pub(crate) span: Span,
    pub(crate) resolved: bool,
    pub(crate) field_class: ASTFieldClass,
}

impl ASTField {
    pub(crate) fn new(source_id: usize, comment_block: Option<CommentBlock>, identifier: ASTIdentifier, r#type: ASTFieldType, decorators: Vec<ASTDecorator>, span: Span) -> Self {
        Self {
            source_id, comment_block, identifier, r#type, decorators, span, resolved: false, field_class: ASTFieldClass::Unresolved,
        }
    }

    pub(crate) fn figure_out_class(&mut self) {
        for decorator in self.decorators.iter() {
            match decorator.expression.as_unit() {
                Some(unit) => {
                    let name = unit.expressions.get(0).unwrap().as_identifier().unwrap().name.as_str();
                    match name {
                        "relation" => {
                            self.field_class = ASTFieldClass::Relation;
                            return;
                        }
                        "getter" | "setter" => {
                            self.field_class = ASTFieldClass::Property;
                            return;
                        }
                        "dropped" => {
                            self.field_class = ASTFieldClass::DroppedField;
                            return;
                        }
                        _ => {}
                    }
                }
                _ => {},
            }
        }
        self.field_class = ASTFieldClass::Field;
    }
}
