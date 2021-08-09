use crate::error::ServiceError;
use crate::{extract::Path, writer::Type};
use serde_json::Value;
use std::convert::{TryFrom, TryInto};
use std::env::VarError;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum ExpectedType {
    Boolean,
    Float,
    SignedInteger,
    UnsignedInteger,
    Text,
    None,
}

impl ExpectedType {
    fn accept<T, M, F>(
        &self,
        value: &Value,
        map: M,
        ext: F,
        disable_try_parse: bool,
    ) -> Result<Type, ServiceError>
    where
        T: FromStr,
        M: FnOnce(T) -> Type,
        F: FnOnce(&Value) -> Option<T>,
    {
        // extract the value
        let result = ext(value);

        // or else a missing value error
        let result = result.ok_or_else(|| ServiceError::PayloadParse(format!("Missing value")));

        // unless conversion is disabled ...
        let result = if !disable_try_parse {
            // ... try to compensate missing value with a conversion
            result.or_else(|e| match value.as_str().map(|s| s.parse()) {
                Some(r) => r.map_err(|_| {
                    ServiceError::Conversion(format!("Failed to convert from: {}", value))
                }),
                None => Err(e),
            })
        } else {
            result
        };

        // convert to Type and return
        result.map(map)
    }

    pub fn convert(
        &self,
        value: &Value,
        path: &Path,
        disable_try_parse: bool,
    ) -> Result<Type, ServiceError> {
        match self {
            ExpectedType::Text => self.accept(
                value,
                Type::String,
                |v| v.as_str().map(ToString::to_string),
                disable_try_parse,
            ),
            ExpectedType::Boolean => {
                self.accept(value, Type::Boolean, |v| v.as_bool(), disable_try_parse)
            }
            ExpectedType::UnsignedInteger => self.accept(
                value,
                Type::UnsignedInteger,
                |v| v.as_u64(),
                disable_try_parse,
            ),
            ExpectedType::SignedInteger => self.accept(
                value,
                Type::SignedInteger,
                |v| v.as_i64(),
                disable_try_parse,
            ),
            ExpectedType::Float => {
                self.accept(value, Type::Float, |v| v.as_f64(), disable_try_parse)
            }
            ExpectedType::None => match value {
                Value::String(s) => Ok(Type::String(s.clone())),
                Value::Bool(b) => Ok(Type::Boolean(*b)),
                Value::Number(n) => n
                    .as_f64()
                    .map(Type::Float)
                    .or_else(|| n.as_i64().map(Type::SignedInteger))
                    .or_else(|| n.as_u64().map(Type::UnsignedInteger))
                    .ok_or_else(|| {
                        ServiceError::PayloadParse(format!(
                            "Unknown numeric type - path: {}, value: {:?}",
                            path.path, n
                        ))
                    }),
                _ => Err(ServiceError::PayloadParse(format!(
                    "Invalid value type selected - path: {}, value: {:?}",
                    path.path, value
                ))),
            },
        }
    }
}

impl TryFrom<String> for ExpectedType {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "bool" | "boolean" => Ok(ExpectedType::Boolean),
            "float" | "number" => Ok(ExpectedType::Float),
            "int" | "integer" => Ok(ExpectedType::SignedInteger),
            "uint" | "unsigned" => Ok(ExpectedType::UnsignedInteger),
            "string" | "text" => Ok(ExpectedType::Text),
            "" | "none" => Ok(ExpectedType::None),
            _ => anyhow::bail!("Unknown type: {}", value),
        }
    }
}

impl TryFrom<Result<String, VarError>> for ExpectedType {
    type Error = anyhow::Error;

    fn try_from(value: Result<String, VarError>) -> Result<Self, Self::Error> {
        value
            .map(Option::Some)
            .or_else(|err| match err {
                VarError::NotPresent => Ok(None),
                err => Err(err),
            })?
            .map_or_else(|| Ok(ExpectedType::None), TryInto::try_into)
    }
}
