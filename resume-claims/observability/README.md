# Observability: OpenTelemetry, Prometheus, Grafana, New Relic

## The Three Pillars + Beyond
- **Metrics**: Aggregated numbers over time (counters, gauges, histograms).
- **Logs**: Discrete events (structured preferred).
- **Traces**: Request flow across services (distributed tracing).
- **Profiles**: CPU, memory, lock contention (increasingly important).

Modern observability emphasizes **correlation** between these signals and **high cardinality** data.

## OpenTelemetry (The Standard)

**Why it matters**: Vendor-neutral instrumentation. You instrument once and can send to Prometheus, Jaeger, New Relic, Datadog, etc. without changing code.

**Key Components**:
- API + SDK for metrics, traces, logs.
- Semantic conventions (standard attribute names).
- Exporters, processors, batching.
- Context propagation (W3C traceparent, baggage).

**Best Practices**:
- Use automatic instrumentation where possible + manual for business logic.
- Proper span attributes and events.
- Sampling strategies (head-based vs tail-based) — critical at scale.
- Resource attributes for service, environment, version, etc.
- Baggage for propagating business context (e.g., user_id, request_id).

**Common Interview Question**:
"How do you control cardinality and cost when using OpenTelemetry at high scale?"

Strong answer covers attribute limits, sampling, exemplar support, and careful semantic convention usage.

## Prometheus + Grafana

**Prometheus Model**:
- Pull-based (or push via Pushgateway for short-lived jobs).
- Multi-dimensional data model with labels (high cardinality danger).
- PromQL for powerful queries.
- Recording rules and alerting rules.
- Service discovery.

**Key Techniques**:
- Histograms vs summaries (use histograms + exemplars when possible).
- `rate()`, `increase()`, `histogram_quantile()`.
- Recording rules to pre-aggregate expensive queries.
- Federation or Thanos/VictoriaMetrics/Cortex for long-term + global view.

**Grafana**:
- Dashboards as code (JSON or Terraform).
- Panels, variables, transformations.
- Alerting (now unified with Grafana Alerting).
- Loki for logs (pair with Prometheus for traces via Tempo).

**Production Advice**:
- Scrape interval vs evaluation interval.
- Cardinality explosion from high-label-count metrics.
- Retention and downsampling strategies.

## New Relic

**Strengths**: Full-stack observability (APM, infrastructure, logs, browser, mobile, synthetic monitoring). Excellent distributed tracing UI. Good anomaly detection and applied intelligence.

**When companies use it**:
- Need fast time-to-value across many languages and technologies.
- Strong focus on customer experience + business metrics correlation.
- Want good out-of-the-box dashboards and alerting.

**Interview Talking Point**:
"I instrumented services with OpenTelemetry and exported to both Prometheus (for custom SLOs and cost control) and New Relic (for rich APM traces and error analysis). This gave us the best of both worlds."

## SLOs, SLIs, and Error Budgets

You should be comfortable discussing:
- Defining SLIs (latency, availability, correctness, throughput).
- Setting realistic SLOs based on user experience.
- Error budget policies and how they drive engineering priorities.
- Using observability data to measure and improve against these.

## Interview-Ready Deep Questions

1. "Your p99 latency looks great in Grafana but users are complaining. How do you investigate?"
2. "How would you design observability for a high-scale event-driven system with many async handoffs?"
3. "Explain the difference between a histogram and a summary in Prometheus, and when you would choose one."
4. "How do you handle observability in a polyglot environment while keeping instrumentation consistent?"
5. "Walk through how you would implement distributed tracing across Python services, Rust components (via PyO3 or separate), and Spark jobs."

## Practical Claim
"Instrumented the entire platform with OpenTelemetry, exposed custom business metrics to Prometheus, built SLO dashboards in Grafana, and used New Relic for deep APM and error analysis. This reduced mean-time-to-detection for production issues by 70% and gave us data-driven error budget discussions with stakeholders."
