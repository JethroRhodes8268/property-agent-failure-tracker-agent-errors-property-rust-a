use property_agent_failure_tracker::{
    agent_loop::{
        decide_dispatch, AgentInput, InspectionReminder, MaintenanceRequest, Severity,
        TenantDocument,
    },
    infrai_client::{ExceptionPayload, InfraiClient},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = AgentInput {
        maintenance: MaintenanceRequest {
            request_id: "req-1042".into(),
            unit_id: "unit-7b".into(),
            summary: "Water near the electrical panel".into(),
            severity: Severity::Urgent,
        },
        documents: vec![TenantDocument {
            document_id: "lease-7b".into(),
            kind: "lease".into(),
            verified: true,
        }],
        reminders: vec![InspectionReminder {
            inspection_id: "inspection-88".into(),
            due_on: "2026-10-02".into(),
            acknowledged: false,
        }],
    };

    match decide_dispatch(&input) {
        Ok(decision) => println!(
            "dispatched {} to {} with {} open reminder(s)",
            decision.request_id, decision.queue, decision.open_reminders
        ),
        Err(failure) => {
            let step = failure.step();
            let payload = ExceptionPayload {
                title: "property-agent step failed",
                message: failure.to_string(),
                level: "error",
                fingerprint: ["property-agent", step],
                exception: format!("{failure:?}"),
                context: &input.maintenance,
            };
            let client = InfraiClient::from_env()?;
            let receipt = client
                .capture(
                    &payload,
                    &format!("{}:{step}", input.maintenance.request_id),
                )
                .await?;
            println!(
                "captured {step} for {}: {receipt}",
                input.maintenance.request_id
            );
        }
    }
    Ok(())
}
