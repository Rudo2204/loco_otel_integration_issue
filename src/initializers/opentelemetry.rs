use async_trait::async_trait;
use axum::Router as AxumRouter;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use loco_rs::{
    Result,
    app::{AppContext, Initializer},
};

pub struct OpenTelemetryInitializer;

#[async_trait]
impl Initializer for OpenTelemetryInitializer {
    fn name(&self) -> String {
        "opentelemetry".to_string()
    }

    async fn before_run(&self, _app_context: &AppContext) -> Result<()> {
        // already initialized in src/bin/main.rs
        Ok(())
    }

    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        let router = router.layer(OtelInResponseLayer::default());
        // .layer(OtelAxumLayer::default());
        Ok(router)
    }
}
