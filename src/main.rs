mod config;
mod error;
mod expected;
mod extract;
mod http;
mod writer;

use crate::{config::ConfigFromEnv, extract::Processor, http::ActixConfig, writer::PostgresWriter};
use actix_web::{middleware, web, App, HttpServer};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
struct Config {
    #[serde(default)]
    pub actix: ActixConfig,
    pub postgresql: writer::Config,
    #[serde(default)]
    pub disable_try_parse: bool,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = Config::from_env()?;
    let writer = PostgresWriter::new(config.postgresql)?;

    let processor = web::Data::new(Processor::new(writer, config.disable_try_parse)?);

    let max_json_payload_size = config.actix.max_json_payload_size;

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::JsonConfig::default().limit(max_json_payload_size))
            .app_data(processor.clone())
            .service(http::forward)
    })
    .bind(config.actix.bind_addr)?
    .run()
    .await?;

    Ok(())
}
