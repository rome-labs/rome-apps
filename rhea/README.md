# Rhea

## Tracing on Otel Telemetry and Logging 

Rhea supports both OpenTelemetry-based tracing and standard output logging.

Tracing is configured via the following environment variables:

- `ENABLE_OTEL_TRACING`: Enables OpenTelemetry tracing. Set this to `true` to activate it.
- `ENABLE_OTEL_METRICS`: Enables OpenTelemetry metrics collection. Set this to `true` to collect metrics.
- `OTLP_RECEIVER_URL`: Specifies the OTLP (OpenTelemetry Protocol) receiver endpoint. 

By default, standard output logging is also enabled. You can toggle this behavior using:

- `ENABLE_STDOUT_LOGGING_ENV`: Set to `false` to disable stdout logging if needed.

The proxy will emit tracing data in JSON format to stdout unless overridden, and will export spans and metrics to the configured OTLP receiver if tracing and metrics are enabled.

### Example Docker Compose Configuration
```yaml
environment:
  ENABLE_OTEL_TRACING: true
  ENABLE_OTEL_METRICS: true
  OTLP_RECEIVER_URL: http://localhost:4317
  ENABLE_STDOUT_LOGGING_ENV: true
```

This setup ensures that all service-level telemetry is collected and available for observability tooling such as Jaeger, Prometheus, or Grafana via the OTLP pipeline.

