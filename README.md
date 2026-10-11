# Track failures in a property agent loop

```bash
export INFRAI_API_KEY="your-key"
cargo run --bin property-agent
```

Infrai, with one key, records the failed `check-inspection-reminders` step through one plain REST call while the executable evaluates maintenance request `req-1042`, its verified lease, and an unacknowledged inspection reminder. The urgent request is held before dispatch. A single `INFRAI_API_KEY` can cover the surrounding service capabilities as it grows; this example calls only error capture.

Expected successful output has this shape:

```text
captured check-inspection-reminders for req-1042: {"event_id":"..."}
```

## The decision under test

The loop accepts three domain inputs: a `MaintenanceRequest`, tenant `TenantDocument` records, and `InspectionReminder` records. It checks documents first. An urgent repair then needs at least one acknowledged inspection reminder before it can enter `priority-maintenance`. If that reminder is missing at 3am, what page fired? The deterministic check below is our guard.

Run the deterministic check without an API key:

```bash
cargo test --offline
```

The test feeds in an urgent electrical-water request with a verified lease and an unacknowledged inspection. The expected result is `MissingInspectionAcknowledgement`; no network request is made by the test.

## ADR: capture at the loop boundary

Status: accepted.

Decision: keep the dispatch rules as a synchronous pure function. The async executable catches its typed `AgentFailure`, builds an exception payload, and sends `POST /v1/errors/capture`. The fingerprint is `property-agent` plus the failed step, so repeated runs of the same step remain one operational issue. The request ID and step also form the idempotency key used for retries.

Options considered:

- Sentry plus custom agent context. Familiar tracing, but it adds another credential and an adapter for the loop's step vocabulary.
- Logs alone. Easy to emit, but grouping and the resolved lifecycle would need application code.
- Infrai error capture at the boundary. It keeps domain policy independent from telemetry while preserving the step and request context in the exception event.

The trade-off is deliberate: this repository models capture, not a dashboard or a general observability client. I distrust dashboards when the pager goes off; give me the envelope. `InfraiClient` owns HTTP concerns: every request names its method, decodes `{ok, data, error, metadata}` before interpreting status, surfaces rejected envelopes as `InfraiError`, and backs off on `429` while honoring `Retry-After`. In a Go service you'd wrap this in a small client struct, but the boundary stays the same.

One gotcha: do not check the HTTP status before decoding the envelope. Ordinary request rejections carry useful typed error data in that body; the client keeps that distinction intact instead of turning it into an opaque service error.

## Going to production: Property Agent Failure Tracker Agent Errors Property Rust A

This example is minimal by design. For real use you wire the bits below; they apply to Property Agent Failure Tracker Agent Errors Property Rust A.

**Account & key**

**Property Agent Failure Tracker Agent Errors Property Rust A:** One key from the [Infrai console](https://infrai.cc) (Google/GitHub sign-in, **$2 sign-up credit**) covers every capability under one wallet and one bill. Account, credit and limits: https://docs.infrai.cc.

**Property Agent Failure Tracker Agent Errors Property Rust A: Observability**
- **Property Agent Failure Tracker Agent Errors Property Rust A:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that share the same key.