use ash_core::{
    CapabilityAuthorityProvenance, CapabilityBinding, CapabilityBindingDependency,
    CapabilityBindingId, CapabilityBindingKind, CapabilityImplementationId, CapabilityInterfaceId,
    ResourceId, ResourceTypeId, Value,
};

#[test]
fn host_capability_binding_carrier_preserves_provider_surface_and_host_authority() {
    let id = CapabilityBindingId::new();
    let binding = CapabilityBinding::host_provider(
        id,
        "workflow-clock",
        CapabilityInterfaceId::new("Clock"),
        "clock",
        vec!["clock.now".to_string()],
    );

    assert_eq!(binding.id, id);
    assert_eq!(binding.name, "workflow-clock");
    assert_eq!(binding.interface.as_str(), "Clock");
    assert!(binding.dependencies.is_empty());
    assert_eq!(
        binding.authority,
        CapabilityAuthorityProvenance::HostAuthority {
            notes: vec!["host provider admitted by runtime".to_string()]
        }
    );
    assert_eq!(
        binding.kind,
        CapabilityBindingKind::HostProvider {
            provider_name: "clock".to_string(),
            admitted_capabilities: vec!["clock.now".to_string()],
        }
    );
}

#[test]
fn implementation_capability_binding_carrier_keeps_dependency_records_only() {
    let resource_id = ResourceId::new();
    let dependency = CapabilityBindingDependency::Resource {
        name: "store".to_string(),
        resource_id,
        type_id: ResourceTypeId::new("KvStore"),
    };
    let capability_dep = CapabilityBindingDependency::Capability {
        name: "clock".to_string(),
        binding_id: CapabilityBindingId::new(),
        interface: CapabilityInterfaceId::new("Clock"),
    };
    let config_dep = CapabilityBindingDependency::Config {
        name: "prefix".to_string(),
        value: Value::String("wf".to_string()),
    };

    let binding = CapabilityBinding::implementation(
        CapabilityBindingId::new(),
        "workflow-kv",
        CapabilityInterfaceId::new("KeyValue"),
        CapabilityImplementationId::new("WorkflowKvImpl"),
        vec![
            dependency.clone(),
            capability_dep.clone(),
            config_dep.clone(),
        ],
    );

    assert_eq!(binding.name, "workflow-kv");
    assert_eq!(binding.interface.as_str(), "KeyValue");
    assert_eq!(
        binding.kind,
        CapabilityBindingKind::Implementation {
            implementation: CapabilityImplementationId::new("WorkflowKvImpl"),
        }
    );
    assert_eq!(
        binding.dependencies,
        vec![dependency, capability_dep, config_dep]
    );
    assert_eq!(
        binding.authority,
        CapabilityAuthorityProvenance::DerivedAuthority {
            dependency_names: vec![
                "store".to_string(),
                "clock".to_string(),
                "prefix".to_string()
            ],
            notes: vec![
                "implementation binding derives only from admitted dependencies".to_string()
            ],
        }
    );
}
