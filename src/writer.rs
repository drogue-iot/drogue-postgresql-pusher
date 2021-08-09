use crate::error::ServiceError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use deadpool::managed::PoolError;
use deadpool_postgres::Pool;
use rust_decimal::Decimal;
use serde::Deserialize;
use tokio_postgres::{
    types::{ToSql, Type as PgType},
    NoTls,
};

#[async_trait]
pub trait Writer<'a> {
    type Insertion: Insertion<'a>;

    fn new_insertion(&self, timestamp: DateTime<Utc>) -> Self::Insertion;
    async fn write(&self, insertion: Self::Insertion) -> Result<(), ServiceError>;
}

pub trait Insertion<'a> {
    fn add_field(self, field: &str, value: Type) -> Self;
    fn add_tag(self, tag: &str, value: Type) -> Self;
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub table: String,
    #[serde(default = "default_time_column")]
    pub time_column: String,
    pub connection: deadpool_postgres::Config,
}

fn default_time_column() -> String {
    "time".to_string()
}

pub struct PostgresWriter {
    pool: Pool,
    table: String,
    time_column: String,
}

impl PostgresWriter {
    pub fn new(config: Config) -> anyhow::Result<PostgresWriter> {
        Ok(Self {
            pool: config.connection.create_pool(NoTls)?,
            table: config.table,
            time_column: config.time_column,
        })
    }

    pub async fn new_insertion(
        &self,
        timestamp: DateTime<Utc>,
    ) -> Result<PostgresInsertion, ServiceError> {
        let fields = vec![self.time_column.clone()];
        let types = vec![PgType::TIMESTAMPTZ];
        let values: Vec<Box<dyn ToSql + Sync>> = vec![Box::new(timestamp)];

        Ok(PostgresInsertion {
            fields,
            types,
            values,
        })
    }

    pub async fn write(&self, insertion: PostgresInsertion) -> Result<(), ServiceError> {
        self.write_con(insertion)
            .await
            .map_err(|err| ServiceError::Target(err.to_string()))
    }

    async fn write_con(
        &self,
        insertion: PostgresInsertion,
    ) -> Result<(), PoolError<tokio_postgres::Error>> {
        let connection = self.pool.get().await?;

        let sql = insertion.make_sql(&self.table);

        let stmt = connection.prepare_typed(&sql, &insertion.types).await?;
        let values: Vec<_> = insertion.values.iter().map(|v| v.as_ref()).collect();
        connection.execute(&stmt, &values).await?;

        Ok(())
    }
}

pub struct PostgresInsertion {
    fields: Vec<String>,
    types: Vec<PgType>,
    values: Vec<Box<dyn ToSql + Sync>>,
}

impl PostgresInsertion {
    pub fn make_sql(&self, table: &str) -> String {
        let mut str = String::with_capacity(8 * 1024);

        str.push_str("INSERT INTO ");
        str.push_str(table);
        str.push_str(" (");

        let mut first = true;
        for field in &self.fields {
            if first {
                first = false;
            } else {
                str.push_str(", ");
            }
            str.push_str(&field);
        }

        str.push_str(") VALUES (");

        for i in 1..=self.values.len() {
            if i > 1 {
                str.push_str(", ");
            }
            str.push_str(&format!("${}", i));
        }

        str.push_str(")");

        str
    }

    fn add_param(mut self, field: String, param: (PgType, Box<dyn ToSql + Sync>)) -> Self {
        self.fields.push(field);
        self.types.push(param.0);
        self.values.push(param.1);

        self
    }

    fn split(value: Type) -> (PgType, Box<dyn ToSql + Sync>) {
        match value {
            Type::Boolean(value) => (PgType::BOOL, Box::new(value)),
            Type::Float(value) => (PgType::FLOAT8, Box::new(value)),
            Type::UnsignedInteger(value) => (PgType::NUMERIC, Box::new(Decimal::from(value))),
            Type::SignedInteger(value) => (PgType::INT8, Box::new(value)),
            Type::String(value) => (PgType::VARCHAR, Box::new(value)),
        }
    }
}

impl Insertion<'_> for PostgresInsertion {
    fn add_field(self, field: &str, value: Type) -> Self {
        self.add_param(field.into(), Self::split(value))
    }

    fn add_tag(self, tag: &str, value: Type) -> Self {
        self.add_param(tag.into(), Self::split(value))
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Boolean(bool),
    Float(f64),
    SignedInteger(i64),
    UnsignedInteger(u64),
    String(String),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sql() {
        let i = PostgresInsertion {
            fields: vec![],
            types: vec![],
            values: vec![],
        };

        let i = i.add_field("field_float", Type::Float(1.23));
        let i = i.add_field("field_u64", Type::UnsignedInteger(42u64));
        let i = i.add_tag("tag_string", Type::String("foo".into()));

        let sql = i.make_sql("table");

        assert_eq!(
            sql,
            r#"INSERT INTO table (field_float, field_u64, tag_string) VALUES ($1, $2, $3)"#
        );
    }
}
