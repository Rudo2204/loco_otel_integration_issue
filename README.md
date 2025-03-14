# Start

`git checkout axum-tracing-opentelemetry`

```
OTEL_SERVICE_NAME="loco_otel_integration_issue" \
OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://localhost:4317" \
OTEL_EXPORTER_OTLP_TRACES_PROTOCOL="grpc" \
OTEL_TRACES_SAMPLER="always_on" \
cargo run -- start
```

# API request

`curl -s localhost:5150/_ping` and `curl -s localhost:5150/api`

# Issue

This is a tracking issue for `opentelemetry-appender-tracing` with `experimental_use_tracing_span_context` feature enabled
when integrating with integrating `axum-tracing-opentelemetry`'s middlewares (`OtelAxumLayer` and `OtelAxumLayer`)
to get more information from the request.

Everything should work correctly without with the default setup. Logs exported have `trace_id`, and they can be used
to correlate logs & traces.

When `axum_tracing_opentelemetry::middleware::OtelAxumLayer` is added to the middleware layers
(by uncommenting it in `src/initializers/opentelemetry.rs`), we do get more information from the request and response,
however `trace_id` is no longer exported.

# Possible causes

I do not think there is a problem with `opentelemetry-appender-tracing`, rather it likely has something to do with
[`tracing-opentelemetry-instrumentation-sdk` creating a new span and setting `trace_id` to `tracing::field::Empty`](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/main/tracing-opentelemetry-instrumentation-sdk/src/http/http_server.rs#L8)
