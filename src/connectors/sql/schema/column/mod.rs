use crate::connectors::sql::dialect::SQLDialect;
use crate::connectors::sql::to_sql_string::ToSQLString;
use crate::core::db_type::DatabaseType;

pub(crate) mod decoder;
pub(crate) mod builder;

#[derive(PartialEq, Clone)]
pub(crate) struct SQLColumn {
    pub(self) name: String,
    pub(self) r#type: DatabaseType,
    pub(self) not_null: bool,
    pub(self) auto_increment: bool,
    pub(self) default: Option<String>,
    pub(self) primary_key: bool,
    pub(self) unique_key: bool,
}

impl SQLColumn {

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn r#type(&self) -> &DatabaseType {
        &self.r#type
    }

    pub(crate) fn not_null(&self) -> bool {
        self.not_null
    }

    pub(crate) fn null(&self) -> bool {
        !self.not_null
    }

    pub(crate) fn auto_increment(&self) -> bool {
        self.auto_increment
    }

    pub(crate) fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }

    pub(crate) fn primary_key(&self) -> bool {
        self.primary_key
    }

    pub(crate) fn unique_key(&self) -> bool {
        self.unique_key
    }
}

impl ToSQLString for SQLColumn {
    fn to_string(&self, dialect: SQLDialect) -> String {
        let name = &self.name;
        let t = self.column_type.to_string(dialect.clone());
        let not_null = if self.not_null { " NOT NULL" } else { " NULL" };
        let primary = if self.primary_key { " PRIMARY KEY" } else { "" };
        let auto_inc = if self.auto_increment { " AUTO_INCREMENT" } else { "" };
        let unique = if self.unique_key { " UNIQUE KEY" } else { "" };
        format!("`{name}` {t}{not_null}{primary}{unique}{auto_inc}")
    }
}