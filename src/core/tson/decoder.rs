use std::collections::{HashSet, HashMap};
use std::str::FromStr;
#[cfg(feature = "data-source-mongodb")]
use bson::oid::ObjectId;
use chrono::{Date, DateTime, NaiveDate, Utc};
use key_path::{KeyPath, path};
use maplit::hashmap;
use rust_decimal::Decimal;
use serde_json::{Value as JsonValue, Map as JsonMap};
use crate::core::action::r#type::ActionType;
use crate::core::error::ActionError;
use crate::core::field::r#type::FieldType;
use crate::core::model::Model;
use crate::core::result::ActionResult;
use crate::core::graph::Graph;
use crate::core::tson::Value;
use crate::tson;

pub(crate) struct Decoder {}

impl Decoder {

    pub(crate) fn decode(model: &Model, graph: &Graph, action: ActionType, json_value: &JsonValue) -> ActionResult<Value> {
        Self::decode_internal(model, graph, action, json_value, path![])
    }

    fn decode_internal(model: &Model, graph: &Graph, action: ActionType, json_value: &JsonValue, path: impl AsRef<KeyPath>) -> ActionResult<Value> {
        let path = path.as_ref();
        let json_map = if let Some(json_map) = json_value.as_object() {
            json_map
        } else {
            return Err(ActionError::unexpected_input_root_type("object"));
        };
        Self::check_json_keys(json_map, action.allowed_input_json_keys(), path)?;
        let mut retval: HashMap<String, Value> = hashmap!{};
        if json_map.contains_key("where") {
            if action.requires_where() {
                retval.insert("where".to_owned(), Self::decode_where(model, graph, json_map.get("where").unwrap(), path + "where")?)
            } else if action.requires_where_unique() {
                retval.insert("where".to_owned(), Self::decode_where_unique(model, graph, json_map.get("where").unwrap(), path + "where")?)
            }
        }
        Ok(Value::HashMap(retval))
    }

    fn check_json_keys<'a>(map: &JsonMap<String, JsonValue>, allowed: &HashSet<&str>, path: &KeyPath<'a>) -> ActionResult<()> {
        if let Some(unallowed) = map.keys().find(|k| !allowed.contains(k.as_str())) {
            return Err(ActionError::unexpected_input_key(unallowed, path + unallowed));
        }
        Ok(())
    }

    fn decode_where(model: &Model, graph: &Graph, json_value: &JsonValue, path: impl AsRef<KeyPath>) -> ActionResult<Value> {
        let path = path.as_ref();
        let json_map = if let Some(json_map) = json_value.as_object() {
            json_map
        } else {
            return Err(ActionError::unexpected_input_type("object", path));
        };
        let mut retval: HashMap<String, Value> = hashmap!{};
        for (key, value) in json_map {
            let key = key.as_str();
            match key {
                "AND" | "OR" => {
                    let path = path + key;
                    match value {
                        JsonValue::Object(_) => {
                            retval.insert(key.to_owned(), Self::decode_where(model, graph, value, path)?)
                        }
                        JsonValue::Array(inner_array) => {
                            retval.insert(key.to_owned(), Value::Vec(inner_array.iter().enumerate().map(|(i, v)| {
                                Self::decode_where(model, graph, v, path + i)?
                            }).collect()?));
                        }
                        _ => {
                            return Err(ActionError::unexpected_input_type("object or array", path));
                        }
                    }
                }
                "NOT" => {
                    let path = path + key;
                    match value {
                        JsonValue::Object(_) => {
                            retval.insert(key.to_owned(), Self::decode_where(model, graph, value, path)?)
                        }
                        _ => {
                            return Err(ActionError::unexpected_input_type("object", path));
                        }
                    }
                }
                _ => {
                    let path = path + key;
                    if !model.query_keys().contains(&key.to_string()) {
                        return Err(ActionError::unexpected_input_key(key, path));
                    }
                    if let Some(field) = model.field(key) {
                        let optional = field.optionality.is_optional();
                        retval.insert(key.to_owned(), Self::decode_where_for_field(graph, field.r#type(), optional, value, path)?);
                    } else if let Some(relation) = model.relation(key) {

                    }
                }
            }
        }
        Ok(Value::HashMap(retval))
    }

    fn decode_where_unique(model: &Model, graph: &Graph, json_value: &JsonValue, path: impl AsRef<KeyPath>) -> ActionResult<Value> {
        let path = path.as_ref();
        let json_map = if let Some(json_map) = json_value.as_object() {
            json_map
        } else {
            return Err(ActionError::unexpected_input_type("object", path));
        };
        if json_map.len() == 0 {
            return Err(ActionError::unexpected_input_value_with_reason("Unique where can't be empty.", path));
        }
        Ok(Value::Null)
    }

    fn decode_where_for_field<'a>(graph: &Graph, r#type: &FieldType, optional: bool, json_value: &JsonValue, path: impl AsRef<KeyPath<'a>>) -> ActionResult<Value> {
        let path = path.as_ref();
        if json_value.is_object() {
            let json_map = json_value.as_object().unwrap();
            Self::check_json_keys(json_map, r#type.filters(), path)?;
            let mut retval: HashMap<String, Value> = hashmap!{};
            for (key, value) in json_map {
                let key = key.as_str();
                let path = path + key;
                match key {
                    "equals" => {
                        retval.insert(key.to_owned(), Self::decode_value_for_field_type(graph, r#type, optional, value, path)?);
                    }
                    "not" => {
                        retval.insert(key.to_owned(), Self::decode_where_for_field(graph, r#type, optional, value, path)?);
                    }
                    "gt" | "gte" | "lt" | "lte" | "contains" | "startsWith" | "endsWith" | "matches" => {
                        retval.insert(key.to_owned(), Self::decode_value_for_field_type(graph, r#type, false, value, path)?);
                    }
                    "in" | "notIn" => {
                        retval.insert(key.to_owned(), Self::decode_value_array_for_field_type(graph, r#type, false, value, path)?);
                    }
                    "mode" => match value.as_str() {
                        Some(s) => if s == "caseInsensitive" {
                            retval.insert(key.to_owned(), Value::String("caseInsensitive".to_owned()));
                        } else {
                            return Err(ActionError::unexpected_input_type("'caseInsensitive'", path));
                        },
                        None => return Err(ActionError::unexpected_input_type("string", path)),
                    }
                    "has" => {
                        let element_field = r#type.element_field().unwrap();
                        retval.insert(key.to_owned(), Self::decode_value_for_field_type(graph, element_field.r#type(), element_field.is_optional(), value, path)?);
                    }
                    "hasEvery" | "hasSome" => {
                        let element_field = r#type.element_field().unwrap();
                        retval.insert(key.to_owned(), Self::decode_value_array_for_field_type(graph, element_field.r#type(), element_field.is_optional(), value, path)?);
                    }
                    "isEmpty" => {
                        retval.insert(key.to_owned(), Self::decode_value_for_field_type(graph, &FieldType::Bool, false, value, path)?);
                    }
                    "length" => {
                        retval.insert(key.to_owned(), Self::decode_value_for_field_type(graph, &FieldType::U64, false, value, path)?);
                    }
                    _ => return Err(ActionError::unexpected_input_key(key, path))
                }
            }
            Ok(Value::HashMap(retval))
        } else {
            Ok(Value::HashMap(hashmap!{"equals" => Self::decode_value_for_field_type(graph, r#type, optional, json_value, path)?}))
        }
    }

    fn decode_value_array_for_field_type<'a>(graph: &Graph, r#type: &FieldType, optional: bool, json_value: &JsonValue, path: impl AsRef<KeyPath<'a>>) -> ActionResult<Value> {
        if let Some(array) = json_value.as_array() {
            Ok(Value::Vec(array.iter().enumerate().map(|(i, v)| {
                Self::decode_value_for_field_type(graph, r#type, optional, v, path + i)?
            }).collect()?))
        } else {
            Err(ActionError::unexpected_input_type("array", path))
        }
    }

    fn decode_value_for_field_type<'a>(graph: &Graph, r#type: &FieldType, optional: bool, json_value: &JsonValue, path: impl AsRef<KeyPath<'a>>) -> ActionResult<Value> {
        if optional && json_value.is_null() {
            return Ok(Value::Null);
        }
        match r#type {
            FieldType::Undefined => panic!("A field cannot have undefined field type"),
            #[cfg(feature = "data-source-mongodb")]
            FieldType::ObjectId => match json_value.as_str() {
                Some(str) => match ObjectId::from_str(str) {
                    Ok(oid) => Ok(Value::ObjectId(oid)),
                    Err(_) => Err(ActionError::unexpected_input_value("object id", path))
                },
                None => Err(ActionError::unexpected_input_type("object id string", path))
            }
            FieldType::Bool => match json_value.as_bool() {
                Some(b) => Ok(Value::Bool(b)),
                None => Err(ActionError::unexpected_input_type("bool", path))
            }
            FieldType::I8 => match json_value.as_i64() {
                Some(i) => Ok(Value::I8(i as i8)),
                None => Err(ActionError::unexpected_input_type("8 bit integer", path))
            }
            FieldType::I16 => match json_value.as_i64() {
                Some(i) => Ok(Value::I16(i as i16)),
                None => Err(ActionError::unexpected_input_type("16 bit integer", path))
            }
            FieldType::I32 => match json_value.as_i64() {
                Some(i) => Ok(Value::I32(i as i32)),
                None => Err(ActionError::unexpected_input_type("32 bit integer", path))
            }
            FieldType::I64 => match json_value.as_i64() {
                Some(i) => Ok(Value::I64(i as i64)),
                None => Err(ActionError::unexpected_input_type("64 bit integer", path))
            }
            FieldType::I128 => match json_value.as_i64() {
                Some(i) => Ok(Value::I128(i as i128)),
                None => match json_value.as_u64() {
                    Some(u) => Ok(Value::I128(u as i128)),
                    None => Err(ActionError::unexpected_input_type("128 bit integer", path))
                }
            }
            FieldType::U8 => match json_value.as_u64() {
                Some(u) => Ok(Value::U8(u as u8)),
                None => Err(ActionError::unexpected_input_type("8 bit unsigned integer", path))
            }
            FieldType::U16 => match json_value.as_u64() {
                Some(u) => Ok(Value::U16(u as u16)),
                None => Err(ActionError::unexpected_input_type("16 bit unsigned integer", path))
            }
            FieldType::U32 => match json_value.as_u64() {
                Some(u) => Ok(Value::U32(u as u32)),
                None => Err(ActionError::unexpected_input_type("32 bit unsigned integer", path))
            }
            FieldType::U64 => match json_value.as_u64() {
                Some(u) => Ok(Value::U64(u as u64)),
                None => Err(ActionError::unexpected_input_type("64 bit unsigned integer", path))
            }
            FieldType::U128 => match json_value.as_u64() {
                Some(u) => Ok(Value::U128(u as u128)),
                None => Err(ActionError::unexpected_input_type("128 bit unsigned integer", path))
            }
            FieldType::F32 => match json_value.as_f64() {
                Some(f) => Ok(Value::F32(f as f32)),
                None => Err(ActionError::unexpected_input_type("32 bit float", path))
            }
            FieldType::F64 => match json_value.as_f64() {
                Some(f) => Ok(Value::F64(f)),
                None => Err(ActionError::unexpected_input_type("64 bit float", path))
            }
            FieldType::Decimal => match json_value.as_str() {
                Some(s) => match Decimal::from_str(s) {
                    Ok(d) => Value::Decimal(d),
                    Err(_) => Err(ActionError::unexpected_input_value("decimal string or float", path))
                }
                None => match json_value.as_f64() {
                    Some(f) => Value::Decimal(Decimal::from(f)),
                    None => Err(ActionError::unexpected_input_value("decimal string or float", path))
                }
            }
            FieldType::String => match json_value.as_str() {
                Some(s) => Value::String(s.to_string()),
                None => Err(ActionError::unexpected_input_value("string", path))
            }
            FieldType::Date => match json_value.as_str() {
                Some(s) => match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    Ok(naive_date) => Ok(Value::Date(Date::from_utc(naive_date, Utc))),
                    Err(_) => Err(ActionError::unexpected_input_value("date string", path))
                }
                None => Err(ActionError::unexpected_input_type("date string", path))
            }
            FieldType::DateTime => match json_value.as_str() {
                Some(s) => match DateTime::parse_from_rfc3339(s) {
                    Ok(fixed_offset_datetime) => Ok(Value::DateTime(fixed_offset_datetime.with_timezone(&Utc))),
                    Err(_) => Err(ActionError::unexpected_input_value("datetime string", path))
                }
                None => Err(ActionError::unexpected_input_type("datetime string", path))
            }
            FieldType::Enum(enum_name) => match json_value.as_str() {
                Some(s) => if graph.enum_values(enum_name.as_str()).unwrap().contains(&s.to_string()) {
                    Ok(Value::String(s.to_string()))
                } else {
                    Err(ActionError::unexpected_input_type(format!("string represents enum {enum_name}"), path))
                },
                None => Err(ActionError::unexpected_input_type(format!("string represents enum {enum_name}"), path))
            }
            FieldType::Vec(inner_field) => match json_value.as_array() {
                Some(a) => {
                    Ok(Value::Vec(a.iter().enumerate().map(|(i, v)| {
                        Self::decode_value_for_field_type(graph, inner_field.r#type(), inner_field.is_optional(), v, path + i)?
                    }).collect()?))
                },
                None => Err(ActionError::unexpected_input_type("array", path))
            }
            FieldType::HashSet(inner_field) => match json_value.as_array() {
                Some(a) => {
                    Ok(Value::HashSet(a.iter().enumerate().map(|(i, v)| {
                        Self::decode_value_for_field_type(graph, inner_field.r#type(), inner_field.is_optional(), v, path + i)?
                    }).collect()?))
                },
                None => Err(ActionError::unexpected_input_type("array", path))
            }
            FieldType::BTreeSet(inner_field) => match json_value.as_array() {
                Some(a) => {
                    Ok(Value::BTreeSet(a.iter().enumerate().map(|(i, v)| {
                        Self::decode_value_for_field_type(graph, inner_field.r#type(), inner_field.is_optional(), v, path + i)?
                    }).collect()?))
                },
                None => Err(ActionError::unexpected_input_type("array", path))
            }
            FieldType::HashMap(inner_field) => match json_value.as_object() {
                Some(a) => {
                    Ok(Value::HashMap(a.iter().map(|(i, v)| {
                        (i.to_string(), Self::decode_value_for_field_type(graph, inner_field.r#type(), inner_field.is_optional(), v, path + i)?)
                    }).collect()?))
                },
                None => Err(ActionError::unexpected_input_type("object", path))
            }
            FieldType::BTreeMap(inner_field) => match json_value.as_object() {
                Some(a) => {
                    Ok(Value::BTreeMap(a.iter().map(|(i, v)| {
                        (i.to_string(), Self::decode_value_for_field_type(graph, inner_field.r#type(), inner_field.is_optional(), v, path + i)?)
                    }).collect()?))
                },
                None => Err(ActionError::unexpected_input_type("object", path))
            }
            FieldType::Object(_) => panic!("Object input is not implemented yet.")
        }
    }
}