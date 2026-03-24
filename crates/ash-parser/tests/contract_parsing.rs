//! Tests for contract and obligation parsing

use ash_parser::parse_workflow::workflow_def;
use ash_parser::input::new_input;

#[test]
fn parse_workflow_with_params() {
    let input = r#"
        workflow transfer(amount: Int, from: Account, to: Account) {
            act do_transfer;
        }
    "#;
    
    let result = workflow_def(&mut new_input(input));
    assert!(result.is_ok(), "Expected workflow with params to parse, got: {:?}", result);
    
    let def = result.unwrap();
    assert_eq!(def.params.len(), 3);
    assert_eq!(def.params[0].name.as_ref(), "amount");
    assert_eq!(def.params[1].name.as_ref(), "from");
    assert_eq!(def.params[2].name.as_ref(), "to");
}

#[test]
fn parse_workflow_with_requires() {
    let input = r#"
        workflow withdraw(amount: Int, account: Account)
            requires: amount > 0
        {
            act do_withdraw;
        }
    "#;
    
    let result = workflow_def(&mut new_input(input));
    assert!(result.is_ok(), "Expected workflow with requires to parse, got: {:?}", result);
    
    let def = result.unwrap();
    assert!(def.contract.is_some(), "Expected contract to be present");
    assert_eq!(def.contract.unwrap().requires.len(), 1);
}

#[test]
fn parse_workflow_with_ensures() {
    let input = r#"
        workflow increment(x: Int)
            ensures: result > 0
        {
            ret x + 1;
        }
    "#;
    
    let result = workflow_def(&mut new_input(input));
    assert!(result.is_ok(), "Expected workflow with ensures to parse, got: {:?}", result);
    
    let def = result.unwrap();
    assert!(def.contract.is_some(), "Expected contract to be present");
    assert_eq!(def.contract.unwrap().ensures.len(), 1);
}

#[test]
fn parse_workflow_with_requires_and_ensures() {
    let input = r#"
        workflow add(x: Int, y: Int)
            requires: x > 0
            ensures: result > x
        {
            ret x + y;
        }
    "#;
    
    let result = workflow_def(&mut new_input(input));
    assert!(result.is_ok(), "Expected workflow with requires and ensures to parse, got: {:?}", result);
    
    let def = result.unwrap();
    assert!(def.contract.is_some(), "Expected contract to be present");
    let contract = def.contract.unwrap();
    assert_eq!(contract.requires.len(), 1);
    assert_eq!(contract.ensures.len(), 1);
}

#[test]
fn parse_oblige_statement() {
    let input = r#"
        workflow example {
            oblige audit_trail;
            act process;
        }
    "#;
    
    let result = workflow_def(&mut new_input(input));
    assert!(result.is_ok(), "Expected workflow with oblige to parse, got: {:?}", result);
}

#[test]
fn parse_check_expression() {
    let input = r#"
        workflow example {
            oblige audit_trail;
            act process;
            let ok = check audit_trail;
            ret ok;
        }
    "#;
    
    let result = workflow_def(&mut new_input(input));
    assert!(result.is_ok(), "Expected workflow with check expression to parse, got: {:?}", result);
}

#[test]
fn parse_check_in_condition() {
    let input = r#"
        workflow example {
            oblige deadline_met;
            act do_work;
            if check deadline_met then {
                ret Success;
            } else {
                act escalate;
                ret Failed;
            }
        }
    "#;
    
    let result = workflow_def(&mut new_input(input));
    assert!(result.is_ok(), "Expected workflow with check in condition to parse, got: {:?}", result);
}

#[test]
fn parse_capability_requirement() {
    let input = r#"
        workflow delete_user(user_id: UserId)
            requires: has_capability(Admin, Operational)
        {
            act do_delete;
        }
    "#;
    
    let result = workflow_def(&mut new_input(input));
    assert!(result.is_ok(), "Expected workflow with capability requirement to parse, got: {:?}", result);
    
    let def = result.unwrap();
    let contract = def.contract.expect("Expected contract");
    assert_eq!(contract.requires.len(), 1);
}

#[test]
fn parse_role_requirement() {
    let input = r#"
        workflow approve_budget(amount: Int)
            requires: has_role(finance_manager)
        {
            act do_approve;
        }
    "#;
    
    let result = workflow_def(&mut new_input(input));
    assert!(result.is_ok(), "Expected workflow with role requirement to parse, got: {:?}", result);
    
    let def = result.unwrap();
    let contract = def.contract.expect("Expected contract");
    assert_eq!(contract.requires.len(), 1);
}

#[test]
fn error_on_invalid_requires() {
    let input = r#"
        workflow example
            requires:
        {
            done;
        }
    "#;
    
    let result = workflow_def(&mut new_input(input));
    assert!(result.is_err(), "Expected empty requires clause to fail");
}

#[test]
fn parse_simple_workflow_without_contract() {
    let input = r#"
        workflow main {
            done;
        }
    "#;
    
    let result = workflow_def(&mut new_input(input));
    assert!(result.is_ok(), "Expected simple workflow to parse, got: {:?}", result);
    
    let def = result.unwrap();
    assert!(def.contract.is_none(), "Expected no contract for simple workflow");
    assert!(def.params.is_empty(), "Expected no params for simple workflow");
}
