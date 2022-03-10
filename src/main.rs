mod config;
mod error;
mod expected;
mod extract;
mod http;
mod writer;

use crate::{config::ConfigFromEnv, extract::Processor, http::ActixConfig, writer::PostgresWriter};
use actix_web::{dev::ServiceRequest, middleware, web, App, Error, HttpServer};
use actix_web_httpauth::{
    extractors::{basic::BasicAuth, bearer::BearerAuth, AuthenticationError},
    headers::www_authenticate::basic::Basic,
    middleware::HttpAuthentication,
};
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Clone, Debug, Deserialize)]
struct Config {
    #[serde(default)]
    pub actix: ActixConfig,
    pub postgresql: writer::Config,
    #[serde(default)]
    pub disable_try_parse: bool,
}

static EMPTY: Cow<'static, str> = Cow::Borrowed("");

async fn basic_auth(req: ServiceRequest, auth: BasicAuth) -> Result<ServiceRequest, Error> {
    let config = req.app_data::<ActixConfig>();

    match config {
        Some(ActixConfig {
            username: Some(username),
            password: Some(password),
            ..
        }) if username == auth.user_id()
            && password == auth.password().as_deref().unwrap_or(&EMPTY) =>
        {
            Ok(req)
        }
        _ => Err(AuthenticationError::new(Basic::new()).into()),
    }
}

async fn bearer_auth(req: ServiceRequest, auth: BearerAuth) -> Result<ServiceRequest, Error> {
    let config = req.app_data::<ActixConfig>();

    match config {
        Some(ActixConfig {
            token: Some(token), ..
        }) if token == auth.token() => Ok(req),
        _ => Err(AuthenticationError::new(Basic::new()).into()),
    }
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = Config::from_env()?;
    let writer = PostgresWriter::new(config.postgresql)?;

    let processor = web::Data::new(Processor::new(writer, config.disable_try_parse)?);

    let max_json_payload_size = config.actix.max_json_payload_size;

    let has_basic = config.actix.username.is_some();
    let has_bearer = config.actix.token.is_some();
    let actix_config = config.actix.clone();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Condition::new(
                has_basic,
                HttpAuthentication::basic(basic_auth),
            ))
            .wrap(middleware::Condition::new(
                has_bearer,
                HttpAuthentication::bearer(bearer_auth),
            ))
            .wrap(middleware::Logger::default())
            .app_data(actix_config.clone())
            .app_data(web::JsonConfig::default().limit(max_json_payload_size))
            .app_data(processor.clone())
            .service(http::forward)
    })
    .bind(config.actix.bind_addr)?
    .run()
    .await?;

    Ok(())
}
