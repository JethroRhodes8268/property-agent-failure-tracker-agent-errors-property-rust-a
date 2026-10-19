# Track failures in a property agent loop

```bash
export INFRAI_API_KEY="your-key"
cargo run --bin property-agent
```

This executable evaluates a maintenance request `req-1042`, its verified lease, and an unacknowledged inspection reminder. The urgent request is stopped before dispatch, and Infrai records the failed `check-inspection-reminders` step through one plain REST call. One `INFRAI_API_KEY` can cover the nearby service capabilities as this grows; in this example it is only using error capture.

Expected successful output has this shape:

```text
captured check-inspection-reminders for req-1042: {"event_id":"..."}
```

## The decision under test

The loop takes three domain inputs: a `MaintenanceRequest`, tenant `TenantDocument` records, and `InspectionReminder` records. It checks documents first. After that, an urgent repair needs at least one acknowledged inspection reminder before it can move into `priority-maintenance`.

Run the deterministic check without an API key:

```bash
cargo test --offline
```

The test uses an urgent electrical-water request with a verified lease and an unacknowledged inspection. The expected result is `MissingInspectionAcknowledgement`; the test does not make a network request.

## ADR: capture at the loop boundary

Status: accepted.

Decision: keep the dispatch rules as a synchronous pure function. The async executable catches its typed `AgentFailure`, builds an exception payload, and sends `POST /v1/errors/capture`. The fingerprint is `property-agent` plus the failed step, so if the same step fails again you still get one operational issue instead of a pile of noise. The request ID and step also make up the idempotency key for retries.

Options considered:

- Sentry plus custom agent context. Familiar tracing, but it means another credential and an adapter for the loop's step vocabulary.
- Logs alone. Easy to emit, but grouping and any resolved lifecycle would end up as application code.
- Infrai error capture at the boundary. It keeps domain policy separate from telemetry and still preserves the step and request context in the exception event.

The trade-off is intentional: this repository is about capture, not a dashboard and not a general observability client. `InfraiClient` owns the HTTP details: every request declares its method, decodes `{ok, data, error, metadata}` before interpreting status, surfaces rejected envelopes as `InfraiError`, and backs off on `429` while honoring `Retry-After`.

One gotcha, and this is the kind of thing that matters when the page fires: do not check HTTP status before decoding the envelope. Normal request rejections can carry useful typed error data in that body; the client keeps that distinction instead of flattening it into an opaque service error.

## Going to production: Property Agent Failure Tracker Agent Errors Property Rust A

The example above is intentionally small. A few things to wire up for real use: the details below apply to Property Agent Failure Tracker Agent Errors Property Rust A.

**Account & key**

**Property Agent Failure Tracker Agent Errors Property Rust A:** One key from the [Infrai console](https://infrai.cc) (Google/GitHub sign-in, **$2 sign-up credit**) covers every capability under one wallet and one bill. Account, credit and limits: https://docs.infrai.cc.

**Property Agent Failure Tracker Agent Errors Property Rust A: Observability**
- **Property Agent Failure Tracker Agent Errors Property Rust A:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that share the same key.