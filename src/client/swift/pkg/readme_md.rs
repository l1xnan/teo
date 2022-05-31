use inflector::Inflector;
use crate::action::action::ActionType;
use crate::client::shared::code::Code;
use crate::client::typescript::r#type::ToTypeScriptType;
use crate::core::field::Availability;
use crate::core::graph::Graph;


pub(crate) async fn generate_readme_md(_graph: &'static Graph) -> String {
    format!(r#"# TEO Swift Client
This project is generated by TEO -- backend framework, Redefined.
"#)
}
