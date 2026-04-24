use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use ash_core::capability::CapabilityError;
use ash_core::{Capability, Constraint, Effect, Expr, Provenance, Value, Workflow};
use ash_interp::act_env::ActEnv;
use ash_interp::behaviour::{MockSettableProvider, TypedSettableProvider};
use ash_interp::capability::MockProvider;
use ash_interp::stream::{MockSendableProvider, StreamContext, TypedSendableProvider};
use ash_interp::{
    BehaviourContext, BehaviourProvider, CapabilityContext, CapabilityProvider, Context,
    PolicyEvaluator, RuntimeState, eval_expr_async, execute_workflow_with_behaviour_in_state,
    execute_workflow_with_stream_in_state,
};
use ash_typeck::types::{Type, TypeVar};
use async_trait::async_trait;

tokio::task_local! {
    static TASK_MARKER: &'static str;
}

#[tokio::test]
async fn act_env_can_be_built_from_runtime_state_and_reuses_existing_capability_context() {
    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(MockProvider::new("sensor", Effect::Epistemic).with_observe_value(Value::Int(7))),
    );
    let provenance = Provenance::new();

    let act_env =
        ActEnv::from_runtime_state(&runtime_state, PolicyEvaluator::new(), provenance.clone())
            .await;

    let observed = act_env
        .capability_ctx
        .observe(&Capability {
            name: "sensor".to_string(),
            effect: Effect::Epistemic,
            constraints: vec![],
        })
        .await
        .expect("capability context should be wired from runtime state");

    assert_eq!(observed, Value::Int(7));
    assert_eq!(act_env.provenance, provenance);
    assert!(act_env.effects.is_empty());
}

#[test]
fn act_env_defaults_to_an_empty_effect_log() {
    let act_env = ActEnv::default();

    assert!(act_env.effects.is_empty());
}

#[derive(Debug)]
struct TokioSleepProvider;

#[derive(Debug)]
struct TaskLocalProvider;

#[derive(Debug)]
struct TaskLocalListProvider;

#[derive(Debug)]
struct CountingProvider {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl CapabilityProvider for TaskLocalProvider {
    fn name(&self) -> &str {
        "tasklocal"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        unreachable!("tasklocal provider does not support observe")
    }

    async fn execute(&self, _action_name: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        let marker = TASK_MARKER.try_with(|value| *value).map_err(|_| {
            CapabilityError::ExecutionFailed("task-local marker missing".to_string())
        })?;
        Ok(Value::String(marker.to_string()))
    }
}

#[async_trait]
impl CapabilityProvider for TaskLocalListProvider {
    fn name(&self) -> &str {
        "tasklist"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        unreachable!("tasklocal list provider does not support observe")
    }

    async fn execute(&self, _action_name: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        let marker = TASK_MARKER.try_with(|value| *value).map_err(|_| {
            CapabilityError::ExecutionFailed("task-local marker missing".to_string())
        })?;
        Ok(Value::List(Box::new(vec![Value::String(
            marker.to_string(),
        )])))
    }
}

#[async_trait]
impl CapabilityProvider for TokioSleepProvider {
    fn name(&self) -> &str {
        "sleepy"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        unreachable!("sleepy provider does not support observe")
    }

    async fn execute(&self, _action_name: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        tokio::time::sleep(Duration::from_millis(1)).await;
        Ok(Value::String("slept".to_string()))
    }
}

#[async_trait]
impl CapabilityProvider for CountingProvider {
    fn name(&self) -> &str {
        "counting"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        unreachable!("counting provider does not support observe")
    }

    async fn execute(&self, _action_name: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Value::String("forced".to_string()))
    }
}

#[tokio::test]
async fn workflow_bridge_attaches_hidden_runtime_act_env_for_expression_forcing() {
    let workflow = Workflow::Ret {
        expr: Expr::FnApply {
            func: Box::new(Expr::Call {
                func: "unit".to_string(),
                module: None,
                arguments: vec![Expr::Literal(Value::Int(7))],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
    };

    let result = execute_workflow_with_behaviour_in_state(
        &workflow,
        Context::new(),
        &CapabilityContext::new(),
        &PolicyEvaluator::new(),
        &BehaviourContext::new(),
        &RuntimeState::new(),
    )
    .await
    .expect("workflow bridge should supply the hidden runtime ActEnv for forcing");

    assert_eq!(
        result,
        Value::List(Box::new(vec![Value::ActEnvToken, Value::Int(7)]))
    );
}

#[tokio::test]
async fn invoke_forced_via_workflow_bridge_dispatches_through_hidden_runtime_actenv() {
    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(
            MockProvider::new("sensor", Effect::Operational)
                .with_execute_result(Ok(Value::String("done".to_string()))),
        ),
    );

    let workflow = Workflow::Ret {
        expr: Expr::FnApply {
            func: Box::new(Expr::Call {
                func: "invoke".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("sensor".to_string())),
                    Expr::Literal(Value::String("read".to_string())),
                    Expr::Literal(Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]))),
                ],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
    };

    let result = execute_workflow_with_behaviour_in_state(
        &workflow,
        Context::new(),
        &CapabilityContext::new(),
        &PolicyEvaluator::new(),
        &BehaviourContext::new(),
        &runtime_state,
    )
    .await
    .expect(
        "invoke should dispatch through the hidden runtime ActEnv when forced from workflow bridge",
    );

    assert_eq!(
        result,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("done".to_string())
        ]))
    );
}

#[tokio::test]
async fn invoke_with_tokio_dependent_provider_dispatches_under_the_simplest_helper_runtime() {
    let runtime_state = RuntimeState::new().with_provider("sleepy", Arc::new(TokioSleepProvider));

    let workflow = Workflow::Ret {
        expr: Expr::FnApply {
            func: Box::new(Expr::Call {
                func: "invoke".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("sleepy".to_string())),
                    Expr::Literal(Value::String("nap".to_string())),
                    Expr::Literal(Value::List(Box::default())),
                ],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
    };

    let result = execute_workflow_with_behaviour_in_state(
        &workflow,
        Context::new(),
        &CapabilityContext::new(),
        &PolicyEvaluator::new(),
        &BehaviourContext::new(),
        &runtime_state,
    )
    .await
    .expect("tokio-dependent provider should run under the simplest helper-runtime bridge");

    assert_eq!(
        result,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("slept".to_string())
        ]))
    );
}

#[tokio::test]
async fn eval_expr_async_force_path_dispatches_tokio_provider_without_helper_runtime() {
    let runtime_state = RuntimeState::new().with_provider("sleepy", Arc::new(TokioSleepProvider));
    let act_env =
        ActEnv::from_runtime_state(&runtime_state, PolicyEvaluator::new(), Provenance::new()).await;
    let ctx = Context::new().with_act_env(act_env);

    let expr = Expr::FnApply {
        func: Box::new(Expr::Call {
            func: "invoke".to_string(),
            module: None,
            arguments: vec![
                Expr::Literal(Value::String("sleepy".to_string())),
                Expr::Literal(Value::String("nap".to_string())),
                Expr::Literal(Value::List(Box::default())),
            ],
        }),
        args: vec![Expr::Literal(Value::ActEnvToken)],
    };

    let result = eval_expr_async(&expr, &ctx)
        .await
        .expect("async force path should dispatch provider without helper-thread bridge");

    assert_eq!(
        result,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("slept".to_string())
        ]))
    );
}

#[tokio::test]
async fn workflow_ret_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));

    let workflow = Workflow::Ret {
        expr: Expr::FnApply {
            func: Box::new(Expr::Call {
                func: "invoke".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("tasklocal".to_string())),
                    Expr::Literal(Value::String("mark".to_string())),
                    Expr::Literal(Value::List(Box::default())),
                ],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
    };

    let result = TASK_MARKER
        .scope(
            "workflow-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("workflow return should use async force path on the current task");

    assert_eq!(
        result,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("workflow-task-local".to_string())
        ]))
    );
}

#[tokio::test]
async fn workflow_ret_unary_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));

    let workflow = Workflow::Ret {
        expr: Expr::Unary {
            op: ash_core::UnaryOp::Not,
            expr: Box::new(Expr::Binary {
                op: ash_core::BinaryOp::Eq,
                left: Box::new(Expr::IndexAccess {
                    expr: Box::new(Expr::FnApply {
                        func: Box::new(Expr::Call {
                            func: "invoke".to_string(),
                            module: None,
                            arguments: vec![
                                Expr::Literal(Value::String("tasklocal".to_string())),
                                Expr::Literal(Value::String("mark".to_string())),
                                Expr::Literal(Value::List(Box::default())),
                            ],
                        }),
                        args: vec![Expr::Literal(Value::ActEnvToken)],
                    }),
                    index: Box::new(Expr::Literal(Value::Int(1))),
                }),
                right: Box::new(Expr::Literal(Value::String("not-the-marker".to_string()))),
            }),
        },
    };

    let result = TASK_MARKER
        .scope(
            "workflow-unary-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("workflow ret unary should use async force path on the current task");

    assert_eq!(result, Value::Bool(true));
}

#[tokio::test]
async fn workflow_let_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));

    let workflow = Workflow::Let {
        pattern: ash_core::Pattern::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        },
        expr: Expr::FnApply {
            func: Box::new(Expr::Call {
                func: "invoke".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("tasklocal".to_string())),
                    Expr::Literal(Value::String("mark".to_string())),
                    Expr::Literal(Value::List(Box::default())),
                ],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
        }),
    };

    let result = TASK_MARKER
        .scope(
            "workflow-let-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("workflow let should use async force path on the current task");

    assert_eq!(
        result,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("workflow-let-task-local".to_string())
        ]))
    );
}

#[tokio::test]
async fn workflow_orient_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));

    let workflow = Workflow::Orient {
        expr: Expr::FnApply {
            func: Box::new(Expr::Call {
                func: "invoke".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("tasklocal".to_string())),
                    Expr::Literal(Value::String("mark".to_string())),
                    Expr::Literal(Value::List(Box::default())),
                ],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::String("continued".to_string())),
        }),
    };

    let result = TASK_MARKER
        .scope(
            "workflow-orient-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("workflow orient should use async force path on the current task");

    assert_eq!(result, Value::String("continued".to_string()));
}

#[tokio::test]
async fn workflow_foreach_uses_async_force_path_for_task_local_provider() {
    let runtime_state =
        RuntimeState::new().with_provider("tasklist", Arc::new(TaskLocalListProvider));

    let workflow = Workflow::ForEach {
        pattern: ash_core::Pattern::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        },
        collection: Expr::FnApply {
            func: Box::new(Expr::Call {
                func: "invoke".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("tasklist".to_string())),
                    Expr::Literal(Value::String("mark".to_string())),
                    Expr::Literal(Value::List(Box::default())),
                ],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
        body: Box::new(Workflow::Ret {
            expr: Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
        }),
    };

    let result = TASK_MARKER
        .scope(
            "workflow-foreach-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("workflow foreach should use async force path on the current task");

    assert_eq!(
        result,
        Value::List(Box::new(vec![Value::String(
            "workflow-foreach-task-local".to_string()
        )]))
    );
}

#[tokio::test]
async fn workflow_check_condition_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));

    let reviewer = ash_core::Role {
        name: "reviewer".to_string(),
        authority: vec![],
        obligations: vec![],
    };

    let workflow = Workflow::Oblig {
        role: reviewer.clone(),
        workflow: Box::new(Workflow::Check {
            obligation: ash_core::Obligation::Obliged {
                role: reviewer,
                condition: Expr::Binary {
                    op: ash_core::BinaryOp::Eq,
                    left: Box::new(Expr::IndexAccess {
                        expr: Box::new(Expr::FnApply {
                            func: Box::new(Expr::Call {
                                func: "invoke".to_string(),
                                module: None,
                                arguments: vec![
                                    Expr::Literal(Value::String("tasklocal".to_string())),
                                    Expr::Literal(Value::String("mark".to_string())),
                                    Expr::Literal(Value::List(Box::default())),
                                ],
                            }),
                            args: vec![Expr::Literal(Value::ActEnvToken)],
                        }),
                        index: Box::new(Expr::Literal(Value::Int(1))),
                    }),
                    right: Box::new(Expr::Literal(Value::String(
                        "workflow-check-task-local".to_string(),
                    ))),
                },
            },
            continuation: Box::new(Workflow::Done),
        }),
    };

    let result = TASK_MARKER
        .scope(
            "workflow-check-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await;

    assert!(
        result.is_ok(),
        "workflow check should evaluate its condition on the current async task: {result:?}"
    );
}

#[tokio::test]
async fn workflow_decide_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));

    let policy =
        ash_interp::Policy::new("tasklocal-decision").with_rule(ash_interp::PolicyRule::new(
            "permit task-local value",
            Expr::Binary {
                op: ash_core::BinaryOp::Eq,
                left: Box::new(Expr::IndexAccess {
                    expr: Box::new(Expr::Variable {
                        name: "decision_value".to_string(),
                        span: ash_core::ast::Span::default(),
                    }),
                    index: Box::new(Expr::Literal(Value::Int(1))),
                }),
                right: Box::new(Expr::Literal(Value::String(
                    "workflow-decide-task-local".to_string(),
                ))),
            },
            ash_core::Decision::Permit,
        ));
    let mut policy_eval = PolicyEvaluator::new();
    policy_eval.register(policy);

    let workflow = Workflow::Decide {
        expr: Expr::FnApply {
            func: Box::new(Expr::Call {
                func: "invoke".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("tasklocal".to_string())),
                    Expr::Literal(Value::String("mark".to_string())),
                    Expr::Literal(Value::List(Box::default())),
                ],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
        policy: "tasklocal-decision".to_string(),
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::String("permitted".to_string())),
        }),
    };

    let result = TASK_MARKER
        .scope(
            "workflow-decide-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &policy_eval,
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("workflow decide should use async force path on the current task");

    assert_eq!(result, Value::String("permitted".to_string()));
}

#[tokio::test]
async fn denied_guard_does_not_force_guarded_provider_act() {
    let calls = Arc::new(AtomicUsize::new(0));
    let runtime_state = RuntimeState::new().with_provider(
        "counting",
        Arc::new(CountingProvider {
            calls: Arc::clone(&calls),
        }),
    );

    let mut policy_eval = PolicyEvaluator::new();
    policy_eval.register(ash_interp::Policy::new("deny-all"));

    let act_env = ActEnv::from_runtime_state(&runtime_state, policy_eval, Provenance::new()).await;
    let ctx = Context::new().with_act_env(act_env);

    let guarded = Expr::FnApply {
        func: Box::new(Expr::Call {
            func: "__guard".to_string(),
            module: Some("act".to_string()),
            arguments: vec![
                Expr::Literal(Value::String("deny-all".to_string())),
                Expr::Call {
                    func: "invoke".to_string(),
                    module: None,
                    arguments: vec![
                        Expr::Literal(Value::String("counting".to_string())),
                        Expr::Literal(Value::String("run".to_string())),
                        Expr::Literal(Value::List(Box::default())),
                    ],
                },
            ],
        }),
        args: vec![Expr::Literal(Value::ActEnvToken)],
    };

    let result = eval_expr_async(&guarded, &ctx)
        .await
        .expect("denied guard should return a denied Act result without forcing the provider");

    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        result,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("policy denied".to_string()),
        ]))
    );
}

#[tokio::test]
async fn workflow_spawn_init_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));
    runtime_state
        .register_child_workflow("worker", Workflow::Done)
        .await;

    let workflow = Workflow::Spawn {
        workflow_type: "worker".to_string(),
        init: Expr::FnApply {
            func: Box::new(Expr::Call {
                func: "invoke".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("tasklocal".to_string())),
                    Expr::Literal(Value::String("mark".to_string())),
                    Expr::Literal(Value::List(Box::default())),
                ],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
        pattern: ash_core::Pattern::Wildcard,
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::String("spawned".to_string())),
        }),
    };

    let result = TASK_MARKER
        .scope(
            "workflow-spawn-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("workflow spawn init should use async force path on the current task");

    assert_eq!(result, Value::String("spawned".to_string()));
}

#[tokio::test]
async fn workflow_set_uses_async_force_path_for_task_local_provider() {
    let provider = MockSettableProvider::new("actuator", "value");
    let typed = TypedSettableProvider::new(provider.clone(), Type::Var(TypeVar::fresh()));
    let mut behaviour_ctx = BehaviourContext::new();
    behaviour_ctx.register_settable(typed);
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));

    let workflow = Workflow::Set {
        capability: "actuator".to_string(),
        channel: "value".to_string(),
        value: Expr::FnApply {
            func: Box::new(Expr::Call {
                func: "invoke".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("tasklocal".to_string())),
                    Expr::Literal(Value::String("mark".to_string())),
                    Expr::Literal(Value::List(Box::default())),
                ],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
    };

    let result = TASK_MARKER
        .scope(
            "workflow-set-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &behaviour_ctx,
                &runtime_state,
            ),
        )
        .await
        .expect("workflow set should use async force path on the current task");

    assert_eq!(result, Value::Null);
    let stored = behaviour_ctx
        .get_settable("actuator", "value")
        .expect("fetch settable provider")
        .sample(&[])
        .await
        .expect("sample set value");
    assert_eq!(
        stored,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("workflow-set-task-local".to_string())
        ]))
    );
}

#[tokio::test]
async fn workflow_send_uses_async_force_path_for_task_local_provider() {
    let provider = MockSendableProvider::new("queue", "output");
    let typed = TypedSendableProvider::new(provider.clone(), Type::Var(TypeVar::fresh()));
    let mut stream_ctx = StreamContext::new();
    stream_ctx.register_sendable(typed);
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));

    let workflow = Workflow::Send {
        capability: "queue".to_string(),
        channel: "output".to_string(),
        value: Expr::FnApply {
            func: Box::new(Expr::Call {
                func: "invoke".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("tasklocal".to_string())),
                    Expr::Literal(Value::String("mark".to_string())),
                    Expr::Literal(Value::List(Box::default())),
                ],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
    };

    let result = TASK_MARKER
        .scope(
            "workflow-send-task-local",
            execute_workflow_with_stream_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &stream_ctx,
                &runtime_state,
            ),
        )
        .await
        .expect("workflow send should use async force path on the current task");

    assert_eq!(result, Value::Null);
    assert_eq!(
        provider.sent_values(),
        vec![Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("workflow-send-task-local".to_string())
        ]))]
    );
}

#[tokio::test]
async fn workflow_yield_request_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));
    {
        let proxy_registry = runtime_state.proxy_registry();
        let mut registry = proxy_registry.lock().await;
        registry.register("reviewer".to_string(), "proxy://reviewer".to_string());
    }

    let workflow = Workflow::Yield {
        role: "reviewer".to_string(),
        request: Box::new(Expr::FnApply {
            func: Box::new(Expr::Call {
                func: "invoke".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("tasklocal".to_string())),
                    Expr::Literal(Value::String("mark".to_string())),
                    Expr::Literal(Value::List(Box::default())),
                ],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        }),
        expected_response_type: ash_core::workflow_contract::TypeExpr::Named("String".to_string()),
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Variable {
                name: "response".to_string(),
                span: ash_core::ast::Span::default(),
            },
        }),
        span: ash_core::ast::Span::default(),
        resume_var: "response".to_string(),
    };

    let result = TASK_MARKER
        .scope(
            "workflow-yield-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await;

    match result {
        Err(ash_interp::error::ExecError::YieldSuspended { request, .. }) => {
            assert_eq!(
                *request,
                Value::List(Box::new(vec![
                    Value::ActEnvToken,
                    Value::String("workflow-yield-task-local".to_string())
                ]))
            );
        }
        other => {
            panic!("workflow yield should suspend with async-evaluated request value: {other:?}")
        }
    }
}

#[tokio::test]
async fn workflow_proxy_resume_response_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));
    let correlation_id = ash_interp::yield_state::CorrelationId::new();
    {
        let suspended = runtime_state.suspended_yields();
        let mut suspended = suspended.lock().await;
        suspended.suspend(ash_interp::yield_state::YieldState {
            correlation_id,
            expected_response_type: ash_typeck::types::Type::String,
            continuation: Workflow::Ret {
                expr: Expr::Variable {
                    name: "response".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            },
            origin_workflow: "workflow-instance".to_string(),
            target_role: "reviewer".to_string(),
            request_sent_at: std::time::Instant::now(),
            resume_var: "response".to_string(),
        });
    }

    let workflow = Workflow::ProxyResume {
        value: Box::new(Expr::FnApply {
            func: Box::new(Expr::Call {
                func: "invoke".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("tasklocal".to_string())),
                    Expr::Literal(Value::String("mark".to_string())),
                    Expr::Literal(Value::List(Box::default())),
                ],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        }),
        value_type: ash_core::workflow_contract::TypeExpr::Named("String".to_string()),
        correlation_id: ash_core::ast::CorrelationId::new(correlation_id.0),
        span: ash_core::ast::Span::default(),
    };

    let result = TASK_MARKER
        .scope(
            "workflow-proxy-resume-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("workflow proxy resume should use async force path on the current task");

    assert_eq!(
        result,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("workflow-proxy-resume-task-local".to_string())
        ]))
    );
}

#[tokio::test]
async fn workflow_split_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));
    runtime_state
        .register_child_workflow("worker", Workflow::Done)
        .await;

    let workflow = Workflow::Split {
        expr: Expr::Let {
            pattern: ash_core::Pattern::Variable {
                name: "marker".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Box::new(Expr::FnApply {
                func: Box::new(Expr::Call {
                    func: "invoke".to_string(),
                    module: None,
                    arguments: vec![
                        Expr::Literal(Value::String("tasklocal".to_string())),
                        Expr::Literal(Value::String("mark".to_string())),
                        Expr::Literal(Value::List(Box::default())),
                    ],
                }),
                args: vec![Expr::Literal(Value::ActEnvToken)],
            }),
            body: Box::new(Expr::Spawn {
                workflow_type: "worker".to_string(),
                init: Box::new(Expr::Literal(Value::Null)),
            }),
            span: ash_core::ast::Span::default(),
        },
        pattern: ash_core::Pattern::Tuple(vec![
            ash_core::Pattern::Wildcard,
            ash_core::Pattern::Variable {
                name: "ctrl".to_string(),
                span: ash_core::ast::Span::default(),
            },
        ]),
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Variable {
                name: "ctrl".to_string(),
                span: ash_core::ast::Span::default(),
            },
        }),
    };

    let result = TASK_MARKER
        .scope(
            "workflow-split-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("workflow split should use async force path on the current task");

    assert!(matches!(result, Value::ControlLink(_)));
}

#[tokio::test]
async fn workflow_ret_constructor_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));

    let workflow = Workflow::Ret {
        expr: Expr::Constructor {
            name: "Tagged".to_string(),
            fields: vec![(
                "value".to_string(),
                Expr::FnApply {
                    func: Box::new(Expr::Call {
                        func: "invoke".to_string(),
                        module: None,
                        arguments: vec![
                            Expr::Literal(Value::String("tasklocal".to_string())),
                            Expr::Literal(Value::String("mark".to_string())),
                            Expr::Literal(Value::List(Box::default())),
                        ],
                    }),
                    args: vec![Expr::Literal(Value::ActEnvToken)],
                },
            )],
        },
    };

    let result = TASK_MARKER
        .scope(
            "workflow-constructor-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("workflow ret constructor should use async force path on the current task");

    assert_eq!(
        result,
        Value::Variant {
            name: "Tagged".to_string(),
            fields: Box::new(vec![(
                "value".to_string(),
                Value::List(Box::new(vec![
                    Value::ActEnvToken,
                    Value::String("workflow-constructor-task-local".to_string()),
                ])),
            )]),
        }
    );
}

#[tokio::test]
async fn workflow_ret_call_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));

    let workflow = Workflow::Ret {
        expr: Expr::Call {
            func: "record".to_string(),
            module: None,
            arguments: vec![
                Expr::Literal(Value::String("value".to_string())),
                Expr::FnApply {
                    func: Box::new(Expr::Call {
                        func: "invoke".to_string(),
                        module: None,
                        arguments: vec![
                            Expr::Literal(Value::String("tasklocal".to_string())),
                            Expr::Literal(Value::String("mark".to_string())),
                            Expr::Literal(Value::List(Box::default())),
                        ],
                    }),
                    args: vec![Expr::Literal(Value::ActEnvToken)],
                },
            ],
        },
    };

    let result = TASK_MARKER
        .scope(
            "workflow-call-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("workflow ret call should use async force path on the current task");

    let mut expected = std::collections::HashMap::new();
    expected.insert(
        "value".to_string(),
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("workflow-call-task-local".to_string()),
        ])),
    );
    assert_eq!(result, Value::Record(Box::new(expected)));
}

#[tokio::test]
async fn workflow_ret_field_access_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));

    let workflow = Workflow::Ret {
        expr: Expr::FieldAccess {
            expr: Box::new(Expr::Call {
                func: "record".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("value".to_string())),
                    Expr::FnApply {
                        func: Box::new(Expr::Call {
                            func: "invoke".to_string(),
                            module: None,
                            arguments: vec![
                                Expr::Literal(Value::String("tasklocal".to_string())),
                                Expr::Literal(Value::String("mark".to_string())),
                                Expr::Literal(Value::List(Box::default())),
                            ],
                        }),
                        args: vec![Expr::Literal(Value::ActEnvToken)],
                    },
                ],
            }),
            field: "value".to_string(),
        },
    };

    let result = TASK_MARKER
        .scope(
            "workflow-field-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("workflow ret field access should use async force path on the current task");

    assert_eq!(
        result,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("workflow-field-task-local".to_string()),
        ]))
    );
}

#[tokio::test]
async fn workflow_ret_match_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));

    let workflow = Workflow::Ret {
        expr: Expr::Match {
            scrutinee: Box::new(Expr::Call {
                func: "record".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("value".to_string())),
                    Expr::FnApply {
                        func: Box::new(Expr::Call {
                            func: "invoke".to_string(),
                            module: None,
                            arguments: vec![
                                Expr::Literal(Value::String("tasklocal".to_string())),
                                Expr::Literal(Value::String("mark".to_string())),
                                Expr::Literal(Value::List(Box::default())),
                            ],
                        }),
                        args: vec![Expr::Literal(Value::ActEnvToken)],
                    },
                ],
            }),
            arms: vec![ash_core::MatchArm {
                pattern: ash_core::Pattern::Record(vec![(
                    "value".to_string(),
                    ash_core::Pattern::Variable {
                        name: "v".to_string(),
                        span: ash_core::ast::Span::default(),
                    },
                )]),
                body: Expr::Variable {
                    name: "v".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            }],
        },
    };

    let result = TASK_MARKER
        .scope(
            "workflow-match-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("workflow ret match should use async force path on the current task");

    assert_eq!(
        result,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("workflow-match-task-local".to_string()),
        ]))
    );
}

#[tokio::test]
async fn workflow_ret_iflet_uses_async_force_path_for_task_local_provider() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));

    let workflow = Workflow::Ret {
        expr: Expr::IfLet {
            pattern: ash_core::Pattern::Record(vec![(
                "value".to_string(),
                ash_core::Pattern::Variable {
                    name: "v".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            )]),
            expr: Box::new(Expr::Call {
                func: "record".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String("value".to_string())),
                    Expr::FnApply {
                        func: Box::new(Expr::Call {
                            func: "invoke".to_string(),
                            module: None,
                            arguments: vec![
                                Expr::Literal(Value::String("tasklocal".to_string())),
                                Expr::Literal(Value::String("mark".to_string())),
                                Expr::Literal(Value::List(Box::default())),
                            ],
                        }),
                        args: vec![Expr::Literal(Value::ActEnvToken)],
                    },
                ],
            }),
            then_branch: Box::new(Expr::Variable {
                name: "v".to_string(),
                span: ash_core::ast::Span::default(),
            }),
            else_branch: Box::new(Expr::Literal(Value::String("nope".to_string()))),
        },
    };

    let result = TASK_MARKER
        .scope(
            "workflow-iflet-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("workflow ret if-let should use async force path on the current task");

    assert_eq!(
        result,
        Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("workflow-iflet-task-local".to_string()),
        ]))
    );
}

fn tasklocal_invoke_force_expr() -> Expr {
    Expr::FnApply {
        func: Box::new(Expr::Call {
            func: "invoke".to_string(),
            module: None,
            arguments: vec![
                Expr::Literal(Value::String("tasklocal".to_string())),
                Expr::Literal(Value::String("mark".to_string())),
                Expr::Literal(Value::List(Box::default())),
            ],
        }),
        args: vec![Expr::Literal(Value::ActEnvToken)],
    }
}

#[tokio::test]
async fn workflow_call_child_body_inherits_hidden_runtime_act_env() {
    let runtime_state = RuntimeState::new().with_provider("tasklocal", Arc::new(TaskLocalProvider));
    runtime_state
        .register_callable_workflow(
            "worker",
            Workflow::Ret {
                expr: tasklocal_invoke_force_expr(),
            },
            0,
            vec![],
        )
        .await;

    let workflow = Workflow::Call {
        target: "worker".to_string(),
        arguments: vec![],
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::String("call continued".to_string())),
        }),
    };

    let result = TASK_MARKER
        .scope(
            "workflow-call-child-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("Workflow::Call child body should inherit hidden runtime ActEnv");

    assert_eq!(result, Value::String("call continued".to_string()));
}

#[tokio::test]
async fn spawned_registered_child_body_has_hidden_runtime_act_env() {
    let runtime_state = RuntimeState::new().with_provider(
        "sensor",
        Arc::new(
            MockProvider::new("sensor", Effect::Operational)
                .with_execute_result(Ok(Value::String("spawned child invoked".to_string()))),
        ),
    );
    runtime_state
        .register_child_workflow(
            "worker",
            Workflow::Ret {
                expr: Expr::FnApply {
                    func: Box::new(Expr::Call {
                        func: "invoke".to_string(),
                        module: None,
                        arguments: vec![
                            Expr::Literal(Value::String("sensor".to_string())),
                            Expr::Literal(Value::String("read".to_string())),
                            Expr::Literal(Value::List(Box::default())),
                        ],
                    }),
                    args: vec![Expr::Literal(Value::ActEnvToken)],
                },
            },
        )
        .await;

    let workflow = Workflow::Spawn {
        workflow_type: "worker".to_string(),
        init: Expr::Literal(Value::Null),
        pattern: ash_core::Pattern::Variable {
            name: "worker".to_string(),
            span: ash_core::ast::Span::default(),
        },
        continuation: Box::new(Workflow::Split {
            expr: Expr::Variable {
                name: "worker".to_string(),
                span: ash_core::ast::Span::default(),
            },
            pattern: ash_core::Pattern::Tuple(vec![
                ash_core::Pattern::Wildcard,
                ash_core::Pattern::Variable {
                    name: "ctrl".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            ]),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "ctrl".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            }),
        }),
    };

    let control = TASK_MARKER
        .scope(
            "spawned-child-task-local",
            execute_workflow_with_behaviour_in_state(
                &workflow,
                Context::new(),
                &CapabilityContext::new(),
                &PolicyEvaluator::new(),
                &BehaviourContext::new(),
                &runtime_state,
            ),
        )
        .await
        .expect("spawn should return a child control link");

    let Value::ControlLink(link) = control else {
        panic!("expected returned control link, got {control:?}");
    };

    let completion = tokio::time::timeout(
        Duration::from_secs(1),
        runtime_state.wait_for_retained_completion(&link),
    )
    .await
    .expect("spawned child should eventually complete")
    .expect("spawned child should seal retained completion");

    assert_eq!(
        completion.terminal_result(),
        Some(&Ok(Value::List(Box::new(vec![
            Value::ActEnvToken,
            Value::String("spawned child invoked".to_string()),
        ]))))
    );
}
