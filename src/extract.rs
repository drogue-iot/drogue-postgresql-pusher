use crate::writer::{Insertion, Type};
use crate::{error::ServiceError, expected::ExpectedType, writer::PostgresWriter};
use chrono::Utc;
use cloudevents::Data;
use cloudevents::{AttributesReader, Event};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct Path {
    pub path: String,
    pub compiled: jsonpath_lib::Compiled,
    pub r#type: ExpectedType,
}

pub struct Processor {
    pub writer: PostgresWriter,
    pub disable_try_parse: bool,
    pub fields: HashMap<String, Path>,
    pub tags: HashMap<String, Path>,
}

impl Processor {
    pub fn new(writer: PostgresWriter, disable_try_parse: bool) -> anyhow::Result<Self> {
        let mut fields = HashMap::new();
        let mut tags = HashMap::new();

        for (key, value) in std::env::vars() {
            if let Some(field) = key.strip_prefix("FIELD_") {
                log::debug!("Adding field - {} -> {}", field, value);
                let compiled = jsonpath_lib::Compiled::compile(&value)
                    .map_err(|err| anyhow::anyhow!("Failed to parse JSON path: {}", err))?;

                // find expected type for the field
                let expected_type = std::env::var(format!("TYPE_FIELD_{}", field)).try_into()?;
                fields.insert(
                    field.to_lowercase(),
                    Path {
                        path: value,
                        compiled,
                        r#type: expected_type,
                    },
                );
            } else if let Some(tag) = key.strip_prefix("TAG_") {
                log::debug!("Adding tag - {} -> {}", tag, value);
                let compiled = jsonpath_lib::Compiled::compile(&value)
                    .map_err(|err| anyhow::anyhow!("Failed to parse JSON path: {}", err))?;

                // find expected type for the tag
                let expected_type = std::env::var(format!("TYPE_TAG_{}", tag)).try_into()?;
                tags.insert(
                    tag.to_lowercase(),
                    Path {
                        path: value,
                        compiled,
                        r#type: expected_type,
                    },
                );
            }
        }

        Ok(Processor {
            writer,
            fields,
            tags,
            disable_try_parse,
        })
    }

    pub async fn process(&self, event: Event) -> Result<usize, ServiceError> {
        let data: Option<&Data> = event.data();
        let json = parse_payload(data)?;
        let timestamp = event.time().cloned().unwrap_or_else(Utc::now);

        let insertion = self.writer.new_insertion(timestamp).await?;

        // process values with payload only

        let (insertion, num) = self.add_values(insertion, &json)?;

        // create full events JSON for tags

        let event_json = serde_json::to_value(event)
            .map_err(|err| ServiceError::PayloadParse(err.to_string()))?;
        let (insertion, _) = self.add_tags(insertion, &event_json)?;

        if num > 0 {
            self.writer.write(insertion).await?;
        }
        Ok(num)
    }

    fn add_values<'a, I>(&self, insertion: I, json: &Value) -> Result<(I, usize), ServiceError>
    where
        I: Insertion<'a>,
    {
        add_to_query(
            insertion,
            self.disable_try_parse,
            &self.fields,
            json,
            |insertion, field, value| insertion.add_field(field, value),
        )
    }

    fn add_tags<'a, I>(&self, insertion: I, json: &Value) -> Result<(I, usize), ServiceError>
    where
        I: Insertion<'a>,
    {
        add_to_query(
            insertion,
            self.disable_try_parse,
            &self.tags,
            json,
            |insertion, field, value| insertion.add_tag(field, value),
        )
    }
}

fn add_to_query<'a, I, F>(
    mut query: I,
    disable_try_parse: bool,
    items: &HashMap<String, Path>,
    json: &Value,
    f: F,
) -> Result<(I, usize), ServiceError>
where
    I: Insertion<'a>,
    F: Fn(I, &String, Type) -> I,
{
    let mut num = 0;

    let mut f = |query, field, value| {
        num += 1;
        f(query, field, value)
    };

    for (ref field, ref path) in items {
        let sel = path
            .compiled
            .select(&json)
            .map_err(|err| ServiceError::Selector(err.to_string()))?;

        query = match sel.as_slice() {
            // no value, don't add
            [] => Ok(query),
            // single value, process
            [v] => Ok(f(
                query,
                field,
                path.r#type.convert(v, path, disable_try_parse)?,
            )),
            // multiple values, error
            [..] => Err(ServiceError::Selector(format!(
                "Selector found more than one value: {}",
                sel.len()
            ))),
        }?;
    }

    Ok((query, num))
}

fn parse_payload(data: Option<&Data>) -> Result<Value, ServiceError> {
    match data {
        Some(Data::Json(value)) => Ok(value.clone()),
        Some(Data::String(s)) => serde_json::from_str::<Value>(&s)
            .map_err(|err| ServiceError::PayloadParse(err.to_string())),

        Some(Data::Binary(b)) => serde_json::from_slice::<Value>(&b)
            .map_err(|err| ServiceError::PayloadParse(err.to_string())),
        _ => Err(ServiceError::PayloadParse(
            "Unknown event payload".to_string(),
        )),
    }
}
