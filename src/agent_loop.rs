use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct MaintenanceRequest {
    pub request_id: String,
    pub unit_id: String,
    pub summary: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Routine,
    Urgent,
}

#[derive(Debug, Clone, Serialize)]
pub struct TenantDocument {
    pub document_id: String,
    pub kind: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectionReminder {
    pub inspection_id: String,
    pub due_on: String,
    pub acknowledged: bool,
}

#[derive(Debug, Clone)]
pub struct AgentInput {
    pub maintenance: MaintenanceRequest,
    pub documents: Vec<TenantDocument>,
    pub reminders: Vec<InspectionReminder>,
}

#[derive(Debug, PartialEq)]
pub struct DispatchDecision {
    pub request_id: String,
    pub queue: &'static str,
    pub open_reminders: usize,
}

#[derive(Debug, Error, PartialEq)]
pub enum AgentFailure {
    #[error("required tenant document is not verified: {document_id}")]
    UnverifiedDocument { document_id: String },
    #[error("urgent request has no acknowledged inspection reminder")]
    MissingInspectionAcknowledgement,
}

impl AgentFailure {
    pub fn step(&self) -> &'static str {
        match self {
            Self::UnverifiedDocument { .. } => "verify-tenant-documents",
            Self::MissingInspectionAcknowledgement => "check-inspection-reminders",
        }
    }
}

pub fn decide_dispatch(input: &AgentInput) -> Result<DispatchDecision, AgentFailure> {
    if let Some(document) = input.documents.iter().find(|document| !document.verified) {
        return Err(AgentFailure::UnverifiedDocument {
            document_id: document.document_id.clone(),
        });
    }

    if matches!(input.maintenance.severity, Severity::Urgent)
        && !input.reminders.iter().any(|reminder| reminder.acknowledged)
    {
        return Err(AgentFailure::MissingInspectionAcknowledgement);
    }

    let queue = match input.maintenance.severity {
        Severity::Routine => "standard-maintenance",
        Severity::Urgent => "priority-maintenance",
    };
    Ok(DispatchDecision {
        request_id: input.maintenance.request_id.clone(),
        queue,
        open_reminders: input
            .reminders
            .iter()
            .filter(|reminder| !reminder.acknowledged)
            .count(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_urgent_dispatch_until_an_inspection_is_acknowledged() {
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

        assert_eq!(
            decide_dispatch(&input),
            Err(AgentFailure::MissingInspectionAcknowledgement)
        );
    }
}
