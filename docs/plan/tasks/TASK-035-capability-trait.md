# TASK-035: Capability Provider Trait

## Status: ✅ Complete

## Description

Define and implement the capability provider trait system for extensible external integrations.

## Specification Reference

- SPEC-004: Operational Semantics - Capability System
- SPEC-001: IR - Capability definitions

## Requirements

### Capability Provider Trait

```rust
/// Trait for capability providers
#[async_trait]
pub trait CapabilityProvider: Send + Sync + Debug {
    /// Get capability metadata
    fn metadata(&self) -> CapabilityMetadata;
    
    /// Execute the capability with given arguments
    async fn execute(&self, args: HashMap<Box<str>, Value>) -> Result<Value, CapabilityError>;
    
    /// Check if provider is available/healthy
    fn is_available(&self) -> bool {
        true
    }
    
    /// Get provider health/status
    fn status(&self) -> ProviderStatus {
        ProviderStatus::Healthy
    }
}

/// Capability metadata
#[derive(Debug, Clone)]
pub struct CapabilityMetadata {
    pub name: Box<str>,
    pub description: Option<Box<str>>,
    pub effect: Effect,
    pub parameters: Vec<ParameterSpec>,
    pub return_type: Type,
    pub category: CapabilityCategory,
}

/// Parameter specification
#[derive(Debug, Clone)]
pub struct ParameterSpec {
    pub name: Box<str>,
    pub ty: Type,
    pub required: bool,
    pub default: Option<Value>,
    pub description: Option<Box<str>>,
}

/// Capability categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityCategory {
    /// File system operations
    Filesystem,
    /// Network operations
    Network,
    /// Database operations
    Database,
    /// External service calls
    ExternalService,
    /// System operations
    System,
    /// Custom/user-defined
    Custom,
}

/// Provider status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderStatus {
    Healthy,
    Degraded,
    Unavailable,
}
```

### Capability Error Types

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum CapabilityError {
    #[error("Missing required argument: {0}")]
    MissingArgument(String),
    
    #[error("Invalid argument type for {name}: expected {expected}, found {actual}")]
    InvalidArgumentType { name: String, expected: String, actual: String },
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Provider unavailable")]
    ProviderUnavailable,
    
    #[error("Custom error: {0}")]
    Custom(String),
}
```

### Capability Registry Enhancements

```rust
/// Enhanced capability registry
#[derive(Debug, Default)]
pub struct CapabilityRegistry {
    providers: HashMap<Box<str>, Arc<dyn CapabilityProvider>>,
    categories: HashMap<CapabilityCategory, Vec<Box<str>>>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn register(&mut self, provider: Arc<dyn CapabilityProvider>) {
        let meta = provider.metadata();
        let name = meta.name.clone();
        let category = meta.category;
        
        self.providers.insert(name.clone(), provider);
        self.categories.entry(category).or_default().push(name);
    }
    
    pub fn get(&self, name: &str) -> Option<Arc<dyn CapabilityProvider>> {
        self.providers.get(name).cloned()
    }
    
    pub fn list(&self) -> Vec<&Box<str>> {
        self.providers.keys().collect()
    }
    
    pub fn list_by_category(&self, category: CapabilityCategory) -> Vec<&Box<str>> {
        self.categories.get(&category)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
    
    pub fn health_check(&self) -> Vec<(Box<str>, ProviderStatus)> {
        self.providers.iter()
            .map(|(name, provider)| (name.clone(), provider.status()))
            .collect()
    }
    
    /// Register all built-in capabilities
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        
        // Filesystem
        registry.register(Arc::new(ReadFileCapability));
        registry.register(Arc::new(WriteFileCapability));
        registry.register(Arc::new(ListDirectoryCapability));
        
        // Network
        registry.register(Arc::new(HttpGetCapability));
        registry.register(Arc::new(HttpPostCapability));
        
        // System
        registry.register(Arc::new(PrintCapability));
        registry.register(Arc::new(EnvVarCapability));
        
        registry
    }
}
```

### Example Capability Implementations

```rust
/// Print to stdout capability
#[derive(Debug)]
pub struct PrintCapability;

#[async_trait]
impl CapabilityProvider for PrintCapability {
    fn metadata(&self) -> CapabilityMetadata {
        CapabilityMetadata {
            name: "print".into(),
            description: Some("Print output to stdout".into()),
            effect: Effect::Epistemic,
            parameters: vec![
                ParameterSpec {
                    name: "message".into(),
                    ty: Type::String,
                    required: true,
                    default: None,
                    description: Some("Message to print".into()),
                },
            ],
            return_type: Type::Null,
            category: CapabilityCategory::System,
        }
    }
    
    async fn execute(&self, args: HashMap<Box<str>, Value>) -> Result<Value, CapabilityError> {
        let message = args.get("message")
            .and_then(|v| match v {
                Value::String(s) => Some(s.as_ref()),
                other => Some(&format!("{:?}", other)),
            })
            .ok_or_else(|| CapabilityError::MissingArgument("message".to_string()))?;
        
        println!("{}", message);
        Ok(Value::Null)
    }
}

/// Environment variable capability
#[derive(Debug)]
pub struct EnvVarCapability;

#[async_trait]
impl CapabilityProvider for EnvVarCapability {
    fn metadata(&self) -> CapabilityMetadata {
        CapabilityMetadata {
            name: "env_var".into(),
            description: Some("Read environment variable".into()),
            effect: Effect::Epistemic,
            parameters: vec![
                ParameterSpec {
                    name: "name".into(),
                    ty: Type::String,
                    required: true,
                    default: None,
                    description: Some("Variable name".into()),
                },
            ],
            return_type: Type::String,
            category: CapabilityCategory::System,
        }
    }
    
    async fn execute(&self, args: HashMap<Box<str>, Value>) -> Result<Value, CapabilityError> {
        let name = args.get("name")
            .and_then(|v| match v {
                Value::String(s) => Some(s.as_ref()),
                _ => None,
            })
            .ok_or_else(|| CapabilityError::MissingArgument("name".to_string()))?;
        
        match std::env::var(name) {
            Ok(val) => Ok(Value::String(val.into_boxed_str())),
            Err(_) => Err(CapabilityError::NotFound(format!("Environment variable: {}", name))),
        }
    }
}

/// List directory capability
#[derive(Debug)]
pub struct ListDirectoryCapability;

#[async_trait]
impl CapabilityProvider for ListDirectoryCapability {
    fn metadata(&self) -> CapabilityMetadata {
        CapabilityMetadata {
            name: "list_directory".into(),
            description: Some("List files in a directory".into()),
            effect: Effect::Epistemic,
            parameters: vec![
                ParameterSpec {
                    name: "path".into(),
                    ty: Type::String,
                    required: true,
                    default: None,
                    description: Some("Directory path".into()),
                },
            ],
            return_type: Type::List(Box::new(Type::String)),
            category: CapabilityCategory::Filesystem,
        }
    }
    
    async fn execute(&self, args: HashMap<Box<str>, Value>) -> Result<Value, CapabilityError> {
        let path = args.get("path")
            .and_then(|v| match v {
                Value::String(s) => Some(std::path::Path::new(s.as_ref())),
                _ => None,
            })
            .ok_or_else(|| CapabilityError::MissingArgument("path".to_string()))?;
        
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(path).await
            .map_err(|e| CapabilityError::IoError(e.to_string()))?;
        
        while let Some(entry) = dir.next_entry().await
            .map_err(|e| CapabilityError::IoError(e.to_string()))? {
            let name = entry.file_name().to_string_lossy().to_string();
            entries.push(Value::String(name.into_boxed_str()));
        }
        
        Ok(Value::List(entries.into()))
    }
}
```

## TDD Steps

### Step 1: Define CapabilityProvider Trait

Create `crates/ash-interp/src/capability.rs`.

### Step 2: Implement Registry Enhancements

Add categories and health checking.

### Step 3: Implement Example Providers

Add PrintCapability, EnvVarCapability, ListDirectoryCapability.

### Step 4: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_print_capability() {
        let cap = PrintCapability;
        let mut args = HashMap::new();
        args.insert("message".into(), Value::String("Hello".into()));
        
        let result = cap.execute(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_env_var_capability() {
        std::env::set_var("TEST_VAR", "test_value");
        
        let cap = EnvVarCapability;
        let mut args = HashMap::new();
        args.insert("name".into(), Value::String("TEST_VAR".into()));
        
        let result = cap.execute(args).await.unwrap();
        assert_eq!(result, Value::String("test_value".into()));
    }

    #[tokio::test]
    async fn test_capability_registry() {
        let mut registry = CapabilityRegistry::new();
        registry.register(Arc::new(PrintCapability));
        
        let provider = registry.get("print");
        assert!(provider.is_some());
        
        let meta = provider.unwrap().metadata();
        assert_eq!(meta.name, "print");
    }

    #[test]
    fn test_registry_builtins() {
        let registry = CapabilityRegistry::with_builtins();
        
        assert!(registry.get("print").is_some());
        assert!(registry.get("read_file").is_some());
        assert!(registry.get("http_get").is_some());
    }
}
```

## Completion Checklist

- [ ] CapabilityProvider trait
- [ ] CapabilityMetadata struct
- [ ] ParameterSpec with defaults
- [ ] CapabilityCategory enum
- [ ] CapabilityError types
- [ ] Enhanced CapabilityRegistry
- [ ] PrintCapability
- [ ] EnvVarCapability
- [ ] ListDirectoryCapability
- [ ] Unit tests for providers
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Extensibility**: Is the trait easy to implement?
2. **Error handling**: Are errors descriptive?
3. **Metadata**: Is all necessary metadata available?

## Estimated Effort

4 hours

## Dependencies

- ash-core: Effect, Value, Type

## Blocked By

- ash-core: Core types

## Blocks

- TASK-033: Operational execution
- All capability-based features
