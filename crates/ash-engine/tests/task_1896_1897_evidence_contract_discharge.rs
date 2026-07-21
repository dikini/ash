//! Phase 194 TASK-1896 and TASK-1897 integration tests.
//!
//! Evidence row substrate and contract-discharge integration for the Ash engine
//! row admission system.

use ash_core::core_ash::{CoreName, CorePath, CoreRow, CoreRowItem};
use ash_core::core_ash_contract::{
    ContractDischargeRecord, ContractDischargeStatus, ContractEvidenceRef, CoreBoundaryId,
};
use ash_core::runtime::ApplicationFailureKind;
use ash_engine::row_admission::{RowAdmissionCheck, RowAdmissionRequirement};
use ash_engine::{ApplicationAdmissionRequest, Engine};

fn base_request(application: &ash_engine::Entry) -> ApplicationAdmissionRequest {
    ApplicationAdmissionRequest {
        entry_name: "contract_evidence".into(),
        body: application.core.clone(),
        application_id: None,
        run_id: None,
        active_role: None,
        admitted_role: None,
        required_capabilities: vec![],
        requires: vec![],
        ensures: vec![],
    }
}

fn core_evidence_row(family: &str, name: &str) -> CoreRowItem {
    let mut path: CorePath = vec![family.into()];
    path.extend(name.split('.').map(CoreName::from));
    CoreRowItem::Evidence { path }
}

#[test]
fn evidence_row_item_carries_family() {
    let row = core_evidence_row("law", "monotonic.bounds");
    let reqs = RowAdmissionRequirement::from_core_row(&CoreRow::closed(vec![row]));
    assert_eq!(reqs.len(), 1);
    let req = reqs.into_iter().next().unwrap();
    match req {
        RowAdmissionRequirement::Evidence { family, evidence } => {
            assert_eq!(family.as_str(), "law");
            assert_eq!(evidence, "law.monotonic.bounds");
        }
        _ => panic!("expected evidence requirement, got {req:?}"),
    }
}

#[test]
fn evidence_row_families_recognized() {
    for family in ["test", "law", "proof", "monitor", "observation"] {
        let row = core_evidence_row(family, "x");
        let reqs = RowAdmissionRequirement::from_core_row(&CoreRow::closed(vec![row]));
        let RowAdmissionRequirement::Evidence { family: got, .. } = reqs[0].clone() else {
            panic!("expected evidence requirement for {family}");
        };
        assert_eq!(got.as_str(), family);
    }
}

#[tokio::test]
async fn evidence_row_rejects_without_record_fail_closed() {
    let engine = Engine::new().build().expect("engine builds");
    let application = Engine::new()
        .build()
        .expect("engine builds")
        .parse(
            "fn checked(n: Int) -> Int where row { evidence test.sorted } { n }\nfn main() { 0 }\n",
        )
        .expect("application parses");
    let request = base_request(&application);

    let outcome = engine
        .admit_application_with_explicit_rows(request, &application)
        .await;

    match outcome {
        ash_engine::ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(
                report.status,
                ash_core::runtime::ApplicationReportStatus::Failed
            );
            assert_eq!(failure.kind, ApplicationFailureKind::RequiresViolation);
            assert!(
                failure
                    .evidence
                    .notes
                    .iter()
                    .any(|note| note.contains("test:test.sorted")),
                "diagnostic should name missing evidence row: {failure:?}"
            );
            assert!(
                failure
                    .evidence
                    .notes
                    .iter()
                    .any(|note| note.contains("rows do not grant authority")),
                "diagnostic should note authority neutrality: {failure:?}"
            );
        }
        other @ ash_engine::ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("expected rejection, got {other:?}")
        }
    }
}

#[tokio::test]
async fn invalid_evidence_family_rejects() {
    let engine = Engine::new().build().expect("engine builds");
    let application = Engine::new()
        .build()
        .expect("engine builds")
        .parse(
            "fn checked(n: Int) -> Int where row { evidence bogus.foo } { n }\nfn main() { 0 }\n",
        )
        .expect("application parses");
    let request = base_request(&application);

    let outcome = engine
        .admit_application_with_explicit_rows(request, &application)
        .await;

    match outcome {
        ash_engine::ApplicationAdmissionOutcome::Rejected { failure, .. } => {
            assert_eq!(failure.kind, ApplicationFailureKind::RequiresViolation);
            assert!(
                failure
                    .evidence
                    .notes
                    .iter()
                    .any(|note| note.contains("invalid family")),
                "diagnostic should reject invalid evidence family: {failure:?}"
            );
        }
        other @ ash_engine::ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("expected rejection, got {other:?}")
        }
    }
}

#[tokio::test]
async fn evidence_row_does_not_grant_authority() {
    let engine = Engine::new().build().expect("engine builds");
    let row = core_evidence_row("monitor", "no_neg");
    let req = RowAdmissionRequirement::from_core_row(&CoreRow::closed(vec![row]))
        .into_iter()
        .next()
        .unwrap();
    let request = base_request(
        &Engine::new()
            .build()
            .expect("engine builds")
            .parse("fn main() { 0 }\n")
            .expect("application parses"),
    );
    let check = RowAdmissionCheck::check(&engine, &request, &req);
    match check {
        RowAdmissionCheck::Missing { kind, notes } => {
            assert_eq!(kind, ApplicationFailureKind::RequiresViolation);
            assert!(
                notes
                    .iter()
                    .any(|note| note.contains("rows do not grant authority"))
            );
        }
        other => panic!("expected Missing, got {other:?}"),
    }
}

#[test]
fn contract_discharge_record_can_be_set() {
    let mut engine = Engine::new().build().expect("engine builds");
    let application = Engine::new()
        .build()
        .expect("engine builds")
        .parse("fn safe(n: Int) -> Int { n }\nfn main() { 0 }\n")
        .expect("application parses");

    let span = ash_core::core_ash::CoreSourceSpan {
        file: None,
        start: 0,
        end: 1,
    };
    let record = ContractDischargeRecord::static_proven(
        "safe",
        CoreBoundaryId::new("fn:safe:requires"),
        ContractEvidenceRef::new("static"),
        span,
        None,
    );

    let previous = engine.set_contract_discharge_for_callable("safe", record.clone(), &application);
    assert!(previous.is_none());

    let stored = engine
        .contract_discharge_record_for_callable("safe", &application)
        .expect("record should be stored");
    assert_eq!(stored.status(), record.status());
    assert!(matches!(
        stored.status(),
        ContractDischargeStatus::StaticProven { .. }
    ));
}

#[test]
fn contract_row_derives_contract_discharge_requirement() {
    let row = CoreRowItem::Contract {
        contract: CoreName::from("safe"),
    };
    let req = RowAdmissionRequirement::from_core_row(&CoreRow::closed(vec![row]))
        .into_iter()
        .next()
        .unwrap();
    match req {
        RowAdmissionRequirement::Unsupported {
            family,
            description,
        } => {
            assert_eq!(family, "contract");
            assert_eq!(
                description,
                "contract row item 'safe' requires a contract-discharge record"
            );
        }
        other => panic!("expected Unsupported contract requirement, got {other:?}"),
    }
}

#[test]
fn contract_row_without_discharge_record_rejects() {
    let engine = Engine::new().build().expect("engine builds");
    let application = Engine::new()
        .build()
        .expect("engine builds")
        .parse("fn main() { 0 }\n")
        .expect("application parses");
    let req = RowAdmissionRequirement::Unsupported {
        family: "contract",
        description: "contract row item 'safe' requires a contract-discharge record".to_string(),
    };
    let check = RowAdmissionCheck::check(&engine, &base_request(&application), &req);
    match check {
        RowAdmissionCheck::Missing { kind, notes } => {
            assert_eq!(kind, ApplicationFailureKind::RequiresViolation);
            assert!(
                notes
                    .iter()
                    .any(|note| note.contains("requires a ContractDischargeRecord")),
                "diagnostic should require discharge record: {notes:?}"
            );
        }
        other => panic!("expected Missing, got {other:?}"),
    }
}
