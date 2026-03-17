# TASK-041: Trace Integrity (Merkle)

## Status: 🟢 Complete

## Description

Implement cryptographic integrity verification for audit trails using Merkle trees, ensuring tamper-evident logs.

## Specification Reference

- SPEC-001: IR - Provenance integrity
- Merkle tree standards for tamper-evident logging

## Requirements

### Merkle Tree

```rust
use sha2::{Sha256, Digest};

/// Merkle tree for integrity verification
#[derive(Debug, Clone)]
pub struct MerkleTree {
    /// Tree levels (bottom to top)
    levels: Vec<Vec<Hash>>,
    /// Root hash
    root: Hash,
}

/// 32-byte hash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn new(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        Self(hasher.finalize().into())
    }
    
    pub fn combine(left: &Hash, right: &Hash) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(left.0);
        hasher.update(right.0);
        Self(hasher.finalize().into())
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl MerkleTree {
    /// Build a Merkle tree from leaf data
    pub fn from_leaves(leaves: &[Vec<u8>]) -> Self {
        if leaves.is_empty() {
            return Self {
                levels: vec![vec![Hash::new(b"")]],
                root: Hash::new(b""),
            };
        }
        
        // Hash leaves
        let leaf_hashes: Vec<Hash> = leaves.iter()
            .map(|data| Hash::new(data))
            .collect();
        
        let mut levels: Vec<Vec<Hash>> = vec![leaf_hashes];
        
        // Build tree bottom-up
        while levels.last().unwrap().len() > 1 {
            let current = levels.last().unwrap();
            let mut next = Vec::new();
            
            for pair in current.chunks(2) {
                let left = &pair[0];
                let right = pair.get(1).unwrap_or(left);
                next.push(Hash::combine(left, right));
            }
            
            levels.push(next);
        }
        
        let root = levels.last().unwrap()[0];
        
        Self { levels, root }
    }
    
    /// Get the root hash
    pub fn root(&self) -> &Hash {
        &self.root
    }
    
    /// Get proof for a leaf at given index
    pub fn proof(&self, index: usize) -> MerkleProof {
        let mut proof = Vec::new();
        let mut idx = index;
        
        for level in &self.levels[..self.levels.len() - 1] {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            
            if sibling_idx < level.len() {
                proof.push((sibling_idx, level[sibling_idx]));
            }
            
            idx /= 2;
        }
        
        MerkleProof { path: proof, root: self.root }
    }
    
    /// Verify leaf at index against root
    pub fn verify(&self, index: usize, leaf_data: &[u8]) -> bool {
        let leaf_hash = Hash::new(leaf_data);
        let proof = self.proof(index);
        proof.verify(index, &leaf_hash, &self.root)
    }
}

/// Merkle proof for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Sibling hashes from leaf to root
    pub path: Vec<(usize, Hash)>,
    /// Expected root
    pub root: Hash,
}

impl MerkleProof {
    /// Verify a proof
    pub fn verify(&self, index: usize, leaf_hash: &Hash, expected_root: &Hash) -> bool {
        let mut current = *leaf_hash;
        let mut idx = index;
        
        for (sibling_idx, sibling_hash) in &self.path {
            current = if idx % 2 == 0 {
                Hash::combine(&current, sibling_hash)
            } else {
                Hash::combine(sibling_hash, &current)
            };
            idx /= 2;
        }
        
        &current == expected_root
    }
}
```

### Integrity Wrapper

```rust
/// Integrity-protected trace store
#[derive(Debug)]
pub struct IntegrityProtectedTrace {
    /// Trace events
    events: Vec<TraceEvent>,
    /// Event hashes (for verification)
    event_hashes: Vec<Hash>,
    /// Merkle tree
    tree: MerkleTree,
    /// Timestamp when tree was built
    sealed_at: Option<DateTime<Utc>>,
}

impl IntegrityProtectedTrace {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            event_hashes: Vec::new(),
            tree: MerkleTree::from_leaves(&[]),
            sealed_at: None,
        }
    }
    
    /// Add an event
    pub fn add_event(&mut self, event: TraceEvent) -> Result<(), IntegrityError> {
        if self.sealed_at.is_some() {
            return Err(IntegrityError::AlreadySealed);
        }
        
        // Serialize event for hashing
        let event_bytes = serde_json::to_vec(&event)
            .map_err(|e| IntegrityError::Serialization(e.to_string()))?;
        
        self.events.push(event);
        self.event_hashes.push(Hash::new(&event_bytes));
        
        // Rebuild tree
        let leaves: Vec<_> = self.event_hashes.iter()
            .map(|h| h.0.to_vec())
            .collect();
        
        self.tree = MerkleTree::from_leaves(&leaves);
        
        Ok(())
    }
    
    /// Seal the trace (prevent further additions)
    pub fn seal(&mut self) {
        self.sealed_at = Some(Utc::now());
    }
    
    /// Get the root hash
    pub fn root_hash(&self) -> &Hash {
        self.tree.root()
    }
    
    /// Verify entire trace integrity
    pub fn verify(&self) -> IntegrityResult {
        if self.events.is_empty() {
            return IntegrityResult::Valid;
        }
        
        // Verify each event
        for (i, event) in self.events.iter().enumerate() {
            let event_bytes = match serde_json::to_vec(event) {
                Ok(b) => b,
                Err(e) => return IntegrityResult::Invalid {
                    index: i,
                    reason: format!("Serialization error: {}", e),
                },
            };
            
            let expected_hash = Hash::new(&event_bytes);
            if expected_hash != self.event_hashes[i] {
                return IntegrityResult::Invalid {
                    index: i,
                    reason: "Hash mismatch".to_string(),
                };
            }
        }
        
        // Verify tree
        let leaves: Vec<_> = self.event_hashes.iter()
            .map(|h| h.0.to_vec())
            .collect();
        let rebuilt = MerkleTree::from_leaves(&leaves);
        
        if rebuilt.root() != self.tree.root() {
            return IntegrityResult::Invalid {
                index: 0,
                reason: "Root hash mismatch".to_string(),
            };
        }
        
        IntegrityResult::Valid
    }
    
    /// Verify a specific event
    pub fn verify_event(&self, index: usize) -> IntegrityResult {
        if index >= self.events.len() {
            return IntegrityResult::Invalid {
                index,
                reason: "Index out of bounds".to_string(),
            };
        }
        
        let event = &self.events[index];
        let event_bytes = match serde_json::to_vec(event) {
            Ok(b) => b,
            Err(e) => return IntegrityResult::Invalid {
                index,
                reason: format!("Serialization error: {}", e),
            },
        };
        
        if self.tree.verify(index, &event_bytes) {
            IntegrityResult::Valid
        } else {
            IntegrityResult::Invalid {
                index,
                reason: "Verification failed".to_string(),
            }
        }
    }
    
    /// Export with integrity proof
    pub fn export_with_proof(&self) -> IntegrityProof {
        IntegrityProof {
            events: self.events.clone(),
            root_hash: self.tree.root().clone(),
            sealed_at: self.sealed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityProof {
    pub events: Vec<TraceEvent>,
    pub root_hash: Hash,
    pub sealed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityResult {
    Valid,
    Invalid { index: usize, reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum IntegrityError {
    #[error("Trace is already sealed")]
    AlreadySealed,
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Verification error: {0}")]
    Verification(String),
}
```

### Digital Signatures

```rust
/// Signed trace for non-repudiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTrace {
    /// The trace data
    pub trace: IntegrityProof,
    /// Signature over root hash
    pub signature: Vec<u8>,
    /// Signer identity
    pub signer: Box<str>,
    /// Signing timestamp
    pub signed_at: DateTime<Utc>,
}

impl SignedTrace {
    /// Sign a trace with an ed25519 key
    pub fn sign(
        trace: IntegrityProof,
        signing_key: &ed25519_dalek::SigningKey,
        signer: &str,
    ) -> Result<Self, IntegrityError> {
        let message = trace.root_hash.0;
        let signature = signing_key.sign(&message);
        
        Ok(Self {
            trace,
            signature: signature.to_bytes().to_vec(),
            signer: signer.into(),
            signed_at: Utc::now(),
        })
    }
    
    /// Verify signature
    pub fn verify(&self, verify_key: &ed25519_dalek::VerifyingKey) -> Result<bool, IntegrityError> {
        let message = self.trace.root_hash.0;
        let signature_bytes: [u8; 64] = self.signature.as_slice()
            .try_into()
            .map_err(|_| IntegrityError::Verification("Invalid signature length".to_string()))?;
        
        let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes);
        
        Ok(verify_key.verify_strict(&message, &signature).is_ok())
    }
}
```

## TDD Steps

### Step 1: Implement Merkle Tree

Create `crates/ash-provenance/src/integrity.rs`.

### Step 2: Implement Integrity Wrapper

Add IntegrityProtectedTrace.

### Step 3: Implement Signatures

Add SignedTrace.

### Step 4: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree() {
        let leaves: Vec<_> = (0..4)
            .map(|i| format!("leaf{}", i).into_bytes())
            .collect();
        
        let tree = MerkleTree::from_leaves(&leaves);
        
        assert!(!tree.root().0.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_merkle_proof() {
        let leaves: Vec<_> = (0..4)
            .map(|i| format!("leaf{}", i).into_bytes())
            .collect();
        
        let tree = MerkleTree::from_leaves(&leaves);
        let proof = tree.proof(1);
        
        let leaf_hash = Hash::new(&leaves[1]);
        assert!(proof.verify(1, &leaf_hash, tree.root()));
    }

    #[test]
    fn test_integrity_protected_trace() {
        let mut trace = IntegrityProtectedTrace::new();
        
        trace.add_event(TraceEvent::WorkflowStarted {
            workflow_id: WorkflowId::new(),
            timestamp: Utc::now(),
            inputs: HashMap::new(),
        }).unwrap();
        
        assert_eq!(trace.verify(), IntegrityResult::Valid);
    }

    #[test]
    fn test_trace_sealing() {
        let mut trace = IntegrityProtectedTrace::new();
        
        trace.add_event(TraceEvent::WorkflowStarted {
            workflow_id: WorkflowId::new(),
            timestamp: Utc::now(),
            inputs: HashMap::new(),
        }).unwrap();
        
        trace.seal();
        
        assert!(trace.add_event(TraceEvent::WorkflowCompleted {
            workflow_id: WorkflowId::new(),
            timestamp: Utc::now(),
            result: Value::Null,
            duration_ms: 0,
        }).is_err());
    }
}
```

## Completion Checklist

- [ ] Hash type with SHA-256
- [ ] MerkleTree
- [ ] MerkleProof
- [ ] IntegrityProtectedTrace
- [ ] IntegrityProof
- [ ] SignedTrace
- [ ] Signature verification
- [ ] Unit tests for Merkle tree
- [ ] Unit tests for integrity
- [ ] Unit tests for signatures
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Security**: Is the cryptographic implementation sound?
2. **Performance**: Is Merkle tree construction efficient?
3. **Tamper evidence**: Would any modification be detectable?

## Estimated Effort

6 hours

## Dependencies

- sha2, ed25519-dalek, hex crates

## Blocked By

- TASK-038: Trace recording
- TASK-039: Lineage tracking

## Blocks

- TASK-055: CLI trace command
