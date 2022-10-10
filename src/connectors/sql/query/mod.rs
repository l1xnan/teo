use crate::connectors::sql::query::alter_table::SQLAlterTableStatement;
use crate::connectors::sql::query::create::SQLCreateStatement;
use crate::connectors::sql::query::delete_from::SQLDeleteFromStatement;
use crate::connectors::sql::query::describe::SQLDescribeStatement;
use crate::connectors::sql::query::drop::SQLDropStatement;
use crate::connectors::sql::query::insert_into::SQLInsertIntoStatement;
use crate::connectors::sql::query::r#use::SQLUseStatement;
use crate::connectors::sql::query::select::SQLSelectStatement;
use crate::connectors::sql::query::show::SQLShowStatement;
use crate::connectors::sql::query::update::SQLUpdateStatement;

pub mod create;
pub mod drop;
pub mod insert_into;
pub mod r#use;
pub mod show;
pub mod describe;
pub mod alter_table;
pub mod select;
pub mod update;
pub mod delete_from;

pub struct SQL { }

impl SQL {
    pub fn create() -> SQLCreateStatement {
        SQLCreateStatement { }
    }

    pub fn drop() -> SQLDropStatement {
        SQLDropStatement { }
    }

    pub fn r#use() -> SQLUseStatement {
        SQLUseStatement { }
    }

    pub fn show() -> SQLShowStatement {
        SQLShowStatement { }
    }

    pub fn describe(table: impl Into<String>) -> SQLDescribeStatement {
        SQLDescribeStatement { table: table.into() }
    }

    pub fn alter_table(table: impl Into<String>) -> SQLAlterTableStatement {
        SQLAlterTableStatement { table: table.into() }
    }

    pub fn insert_into(table: &str) -> SQLInsertIntoStatement {
        SQLInsertIntoStatement { table, values: vec![] }
    }

    pub fn update(table: &str) -> SQLUpdateStatement {
        SQLUpdateStatement { table, values: vec![], r#where: "" }
    }

    pub fn delete_from(from: &str) -> SQLDeleteFromStatement {
        SQLDeleteFromStatement { from, r#where: None }
    }

    pub fn select<'a>(columns: Option<&'a Vec<&'a str>>, from: &'a str) -> SQLSelectStatement<'a> {
        SQLSelectStatement { columns, from, r#where: None, order_by: None, limit: None, left_join: None }
    }
}