use crate::extract::Processor;
use actix_web::{post, web, HttpResponse};
use cloudevents::Event;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ActixConfig {
    #[serde(default = "default_max_json_payload_size")]
    pub max_json_payload_size: usize,
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,

    pub username: Option<String>,
    pub password: Option<String>,

    pub token: Option<String>,
}

impl Default for ActixConfig {
    fn default() -> Self {
        Self {
            max_json_payload_size: default_max_json_payload_size(),
            bind_addr: default_bind_addr(),
            username: None,
            password: None,
            token: None,
        }
    }
}

#[inline]
fn default_bind_addr() -> String {
    "127.0.0.1:8080".into()
}

#[inline]
fn default_max_json_payload_size() -> usize {
    64 * 1024
}

#[post("/")]
pub async fn forward(
    event: Event,
    processor: web::Data<Processor>,
) -> Result<HttpResponse, actix_web::Error> {
    log::debug!("Received Event: {:?}", event);

    Ok(match processor.process(event).await? {
        0 => HttpResponse::NoContent().finish(),
        _ => HttpResponse::Accepted().finish(),
    })
}
