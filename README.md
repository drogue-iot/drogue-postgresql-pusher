# Pushing Cloud Events to PostreSQL (and TimescaleDB)

[![CI](https://github.com/drogue-iot/drogue-postgresql-pusher/workflows/CI/badge.svg)](https://github.com/drogue-iot/drogue-postgresql-pusher/actions?query=workflow%3A%22CI%22)
[![GitHub release (latest SemVer)](https://img.shields.io/github/v/tag/drogue-iot/drogue-postgresql-pusher?sort=semver)](https://github.com/orgs/drogue-iot/packages/container/package/drogue-postgresql-pusher)
[![Matrix](https://img.shields.io/matrix/drogue-iot:matrix.org)](https://matrix.to/#/#drogue-iot:matrix.org)

Extracts information from JSON based cloud events and pushes them to PostgreSQL. It is intended to be used with
TimescaleDB.

## Input

Cloud event:

* **Data Content Type**: Mime type of the payload, must be `application/json`
* **Payload**: JSON payload from which to extract values.

## Output

There is no output. The result will be written to the configured PostgreSQL instance.

## Payload

The application expects a JSON payload structure, from which it extracts fields and tags using *JSON path* expressions.

## Configuration

You can use the following environment variables to configure its behavior:

| Name | Required | Default | Description |
| ---- | -------- | ------- | ----------- |
| `DISABLE_TRY_PARSE` | | `false` | Disable trying to parse expected value from String format |
| `RUST_LOG` | | none | The configuration of the logger, also see https://docs.rs/env_logger/latest/env_logger/ |
| `ACTIX__BIND_ADDR` | | `127.0.0.1:8080` | The address the HTTP server binds to |
| `ACTIX__MAX_JSON_PAYLOAD_SIZE` | | `65536` | Maximum payload size for JSON |
| `POSTGRESQL__TABLE` | x | none | The table to write to |
| `POSTGRESQL__TIME_COLUMN` | x | none | The column to receive the timestamp |
| `POSTGRESQL__CONNECTION__HOST` | x | none | The hostname (or IP address) of the PostgreSQL instance |
| `POSTGRESQL__CONNECTION__USER` | x | none | The username to use for authenticating to the database |
| `POSTGRESQL__CONNECTION__PASSWORD` | x | none | The password to use for authenticating to the database |
| `POSTGRESQL__CONNECTION__DBNAME` | x | none | The database to use |

#### Tags and fields

Additionally, you need to configure a set of fields and (optionally) some tags, which make up the write query. Both
are configured using environment variables. Fields are prefixed with `FIELD_` and tags are prefixed with `TAG_`.

JSON paths for both fields and tags must result in a single element. Queries which end up with no fields will not
be executed.

Paths for fields are rooted to the data section of the cloud event. Paths for tags are rooted at the JSON
representation of the cloud event.

#### Value types

You can also add a `TYPE_FIELD_` (and `TYPE_TAG_`) prefixed variables, which define the expected type for the field
or tag.

The following types are available:

<dl>
    <dt><code>none</code> (the default)</dt> <dd>Try auto-conversion. For numbers, this will try a float first, then fall back to signed, and then to unsigned integers.</dd>
    <dt><code>float</code>, <code>number</code></dt> <dd>Floating point value (`DOUBLE PRECISION`)</dd>
    <dt><code>string</code>, <code>text</code></dt> <dd>Text value (`VARCHAR`)</dd>
    <dt><code>bool</code>, <code>boolean</code></dt> <dd>Boolean value (`BOOLEAN`)</dd>
    <dt><code>int</code>, <code>integer</code></dt> <dd>Signed integer value (`BIGINT`)</dd>
    <dt><code>uint</code>, <code>unsigned</code></dt> <dd>Unsigned integer value (`NUMERIC`)</dd>
</dl>

If a value cannot be converted, and error is raised.

#### PostgreSQL specifics

For PostgresSQL, tags and fields will end up in the same SQL statement, simply adding them as an SQL field in the
insert statement. The only difference is, that tags have access to the full cloud events JSON for extracting
information, and fields have not.

### Examples

The following example defines a field (named `temperature`), which will take the value from the field `temp` of the
data section of the cloud events:

~~~yaml
- name: FIELD_TEMPERATURE
  value: $.temp
~~~

For each field, you can also configure the expected type, the default is to try and auto-convert the value:

~~~yaml
- name: TYPE_FIELD_TEMPERATURE
  value: float
~~~


The following example defines a tag (named `device_id`), which will take the value from the cloud events attribute
`subject`:

~~~yaml
- name: TAG_DEVICE_ID
  value: $.subject
~~~

## Building

You can build the container image using:

~~~shell
cargo build --release
docker build . -t drogue-influxdb-pusher
~~~
