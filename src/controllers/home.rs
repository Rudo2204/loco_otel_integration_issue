use axum::debug_handler;
use loco_rs::prelude::*;

use crate::views::home::HomeResponse;

#[debug_handler]
#[tracing::instrument]
async fn current() -> Result<Response> {
    tracing::info!("hello loco");
    format::json(HomeResponse::new("loco"))
}

pub fn routes() -> Routes {
    Routes::new().prefix("/api").add("/", get(current))
}
