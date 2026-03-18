//! Integrity verification for audit trails
//!
//! This module provides cryptographic hashing and Merkle tree structures
//! for tamper-evident logging of provenance data.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// A 32-byte SHA-256 hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Hash([u8; 32]);

impl Hash {
    /// Create a new hash from raw bytes.
    pub const fn new(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }

    /// Create a hash from a byte slice using SHA-256.
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        Hash(hasher.finalize().into())
    }

    /// Create a hash from a string.
    pub fn from_string(data: &str) -> Self {
        Self::from_bytes(data.as_bytes())
    }

    /// Get the raw bytes of the hash.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to a hexadecimal string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse a hash from a hexadecimal string.
    ///
    /// # Errors
    ///
    /// Returns an error if the hex string is invalid or the wrong length.
    pub fn from_hex(hex: &str) -> Result<Self, IntegrityError> {
        let bytes = hex::decode(hex).map_err(|e| IntegrityError::InvalidHex(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(IntegrityError::InvalidLength(bytes.len()));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Hash(array))
    }

    /// Hash two hashes together (used in Merkle trees).
    pub fn combine(left: &Self, right: &Self) -> Self {
        let mut data = Vec::with_capacity(64);
        data.extend_from_slice(&left.0);
        data.extend_from_slice(&right.0);
        Self::from_bytes(&data)
    }

    /// Check if this hash is all zeros (placeholder).
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(bytes)
    }
}

/// Errors that can occur in integrity operations.
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum IntegrityError {
    /// Invalid hexadecimal string.
    #[error("invalid hex: {0}")]
    InvalidHex(String),
    /// Invalid hash length.
    #[error("invalid hash length: expected 32, got {0}")]
    InvalidLength(usize),
    /// Merkle tree is empty.
    #[error("merkle tree is empty")]
    EmptyTree,
    /// Invalid proof.
    #[error("invalid proof")]
    InvalidProof,
    /// Index out of bounds.
    #[error("index out of bounds: {0}")]
    IndexOutOfBounds(usize),
    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// A Merkle tree for tamper-evident logging.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MerkleTree {
    /// The leaf nodes (hashes of original data).
    leaves: Vec<Hash>,
    /// The tree levels, from leaves (level 0) to root.
    levels: Vec<Vec<Hash>>,
    /// The root hash of the tree.
    root: Option<Hash>,
}

impl MerkleTree {
    /// Create a new empty Merkle tree.
    pub fn new() -> Self {
        Self {
            leaves: Vec::new(),
            levels: Vec::new(),
            root: None,
        }
    }

    /// Create a Merkle tree from a list of data items.
    pub fn from_data<T: AsRef<[u8]>>(data: &[T]) -> Self {
        let mut tree = Self::new();
        for item in data {
            tree.push(item);
        }
        tree
    }

    /// Create a Merkle tree from pre-computed hashes.
    pub fn from_hashes(hashes: Vec<Hash>) -> Self {
        let mut tree = Self::new();
        tree.leaves = hashes;
        tree.rebuild();
        tree
    }

    /// Add a new data item to the tree.
    pub fn push<T: AsRef<[u8]>>(&mut self, data: T) {
        let hash = Hash::from_bytes(data.as_ref());
        self.leaves.push(hash);
        self.rebuild();
    }

    /// Add a pre-computed hash to the tree.
    pub fn push_hash(&mut self, hash: Hash) {
        self.leaves.push(hash);
        self.rebuild();
    }

    /// Get the root hash of the tree.
    pub fn root(&self) -> Option<Hash> {
        self.root
    }

    /// Get the number of leaves.
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Check if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// Get the leaf at a specific index.
    pub fn get_leaf(&self, index: usize) -> Option<Hash> {
        self.leaves.get(index).copied()
    }

    /// Rebuild the tree structure.
    fn rebuild(&mut self) {
        if self.leaves.is_empty() {
            self.levels.clear();
            self.root = None;
            return;
        }

        self.levels.clear();
        self.levels.push(self.leaves.clone());

        // Build tree level by level
        while self.levels.last().unwrap().len() > 1 {
            let current_level = self.levels.last().unwrap();
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(Hash::combine(&chunk[0], &chunk[1]));
                } else {
                    // Odd number of nodes: duplicate the last one
                    next_level.push(Hash::combine(&chunk[0], &chunk[0]));
                }
            }

            self.levels.push(next_level);
        }

        self.root = self.levels.last().and_then(|l| l.first()).copied();
    }

    /// Generate a proof for a specific leaf index.
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds.
    pub fn generate_proof(&self, index: usize) -> Result<MerkleProof, IntegrityError> {
        if index >= self.leaves.len() {
            return Err(IntegrityError::IndexOutOfBounds(index));
        }

        let mut path = Vec::new();
        let mut current_index = index;

        for level in &self.levels {
            if level.len() <= 1 {
                break;
            }

            let sibling_index = if current_index.is_multiple_of(2) {
                // Left node: sibling is to the right
                if current_index + 1 < level.len() {
                    current_index + 1
                } else {
                    // Odd length: sibling is self
                    current_index
                }
            } else {
                // Right node: sibling is to the left
                current_index - 1
            };

            path.push(level[sibling_index]);
            current_index /= 2;
        }

        Ok(MerkleProof {
            leaf_index: index,
            path,
            root: self.root.ok_or(IntegrityError::EmptyTree)?,
        })
    }

    /// Verify a proof for a specific leaf.
    pub fn verify_proof(&self, proof: &MerkleProof, leaf_hash: Hash) -> bool {
        verify_integrity(leaf_hash, proof)
    }

    /// Get all leaf hashes.
    pub fn leaves(&self) -> &[Hash] {
        &self.leaves
    }

    /// Clear all data from the tree.
    pub fn clear(&mut self) {
        self.leaves.clear();
        self.levels.clear();
        self.root = None;
    }
}

impl Default for MerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

/// A proof of inclusion in a Merkle tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleProof {
    /// The index of the leaf in the original tree.
    pub leaf_index: usize,
    /// The sibling hashes along the path to the root.
    pub path: Vec<Hash>,
    /// The expected root hash.
    pub root: Hash,
}

impl MerkleProof {
    /// Create a new Merkle proof.
    pub fn new(leaf_index: usize, path: Vec<Hash>, root: Hash) -> Self {
        Self {
            leaf_index,
            path,
            root,
        }
    }

    /// Verify this proof for a given leaf hash.
    pub fn verify(&self, leaf_hash: Hash) -> bool {
        verify_integrity(leaf_hash, self)
    }
}

/// Generate an integrity proof for a given leaf hash.
///
/// This is a convenience function that creates a Merkle proof for the given
/// leaf index in the provided tree.
///
/// # Errors
///
/// Returns an error if the tree is empty or the index is out of bounds.
pub fn integrity_proof(
    tree: &MerkleTree,
    leaf_index: usize,
) -> Result<MerkleProof, IntegrityError> {
    tree.generate_proof(leaf_index)
}

/// Verify the integrity of a leaf hash against a Merkle proof.
///
/// Returns true if the leaf hash is valid and the proof is correct.
pub fn verify_integrity(leaf_hash: Hash, proof: &MerkleProof) -> bool {
    let mut current_hash = leaf_hash;
    let mut current_index = proof.leaf_index;

    for sibling_hash in &proof.path {
        current_hash = if current_index.is_multiple_of(2) {
            // Left node: combine (current, sibling)
            Hash::combine(&current_hash, sibling_hash)
        } else {
            // Right node: combine (sibling, current)
            Hash::combine(sibling_hash, &current_hash)
        };
        current_index /= 2;
    }

    current_hash == proof.root
}

/// Compute a hash of a serializable value.
///
/// # Errors
///
/// Returns an error if serialization fails.
pub fn hash_value<T: Serialize>(value: &T) -> Result<Hash, IntegrityError> {
    let json = serde_json::to_vec(value)
        .map_err(|e| IntegrityError::Serialization(format!("failed to serialize value: {e}")))?;
    Ok(Hash::from_bytes(&json))
}

/// A tamper-evident log using a Merkle tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TamperEvidentLog {
    tree: MerkleTree,
    entries: Vec<Vec<u8>>,
}

impl TamperEvidentLog {
    /// Create a new empty log.
    pub fn new() -> Self {
        Self {
            tree: MerkleTree::new(),
            entries: Vec::new(),
        }
    }

    /// Append an entry to the log.
    pub fn append<T: AsRef<[u8]>>(&mut self, entry: T) -> Hash {
        let data = entry.as_ref().to_vec();
        let hash = Hash::from_bytes(&data);
        self.entries.push(data);
        self.tree.push_hash(hash);
        hash
    }

    /// Append a serializable entry to the log.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn append_serializable<T: Serialize>(&mut self, entry: &T) -> Result<Hash, IntegrityError> {
        let json = serde_json::to_vec(entry).map_err(|e| {
            IntegrityError::Serialization(format!("failed to serialize entry: {e}"))
        })?;
        Ok(self.append(json))
    }

    /// Get the current root hash.
    pub fn root(&self) -> Option<Hash> {
        self.tree.root()
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get an entry by index.
    pub fn get(&self, index: usize) -> Option<&[u8]> {
        self.entries.get(index).map(|v| v.as_slice())
    }

    /// Generate a proof for a specific entry.
    pub fn prove(&self, index: usize) -> Result<MerkleProof, IntegrityError> {
        self.tree.generate_proof(index)
    }

    /// Verify an entry against a proof.
    pub fn verify(&self, index: usize, proof: &MerkleProof) -> bool {
        if let Some(entry) = self.get(index) {
            let hash = Hash::from_bytes(entry);
            verify_integrity(hash, proof)
        } else {
            false
        }
    }

    /// Get all entries.
    pub fn entries(&self) -> &[Vec<u8>] {
        &self.entries
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.tree.clear();
    }
}

impl Default for TamperEvidentLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_from_bytes() {
        let data = b"hello world";
        let hash = Hash::from_bytes(data);
        assert_eq!(hash.as_bytes().len(), 32);
    }

    #[test]
    fn test_hash_from_string() {
        let hash1 = Hash::from_string("hello");
        let hash2 = Hash::from_string("hello");
        let hash3 = Hash::from_string("world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_to_hex() {
        let hash = Hash::from_string("test");
        let hex = hash.to_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_from_hex() {
        let original = Hash::from_string("test");
        let hex = original.to_hex();
        let restored = Hash::from_hex(&hex).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_hash_from_hex_invalid() {
        let result = Hash::from_hex("invalid");
        assert!(result.is_err());

        let result = Hash::from_hex("abcd");
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_combine() {
        let h1 = Hash::from_string("left");
        let h2 = Hash::from_string("right");
        let combined = Hash::combine(&h1, &h2);

        // Combined should be different from inputs
        assert_ne!(combined, h1);
        assert_ne!(combined, h2);

        // Order matters
        let combined2 = Hash::combine(&h2, &h1);
        assert_ne!(combined, combined2);
    }

    #[test]
    fn test_hash_is_zero() {
        let zero = Hash::default();
        assert!(zero.is_zero());

        let non_zero = Hash::from_string("test");
        assert!(!non_zero.is_zero());
    }

    #[test]
    fn test_merkle_tree_empty() {
        let tree = MerkleTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.root(), None);
    }

    #[test]
    fn test_merkle_tree_single_leaf() {
        let mut tree = MerkleTree::new();
        tree.push("data");

        assert_eq!(tree.len(), 1);
        assert!(tree.root().is_some());
    }

    #[test]
    fn test_merkle_tree_multiple_leaves() {
        let mut tree = MerkleTree::new();
        tree.push("a");
        tree.push("b");
        tree.push("c");

        assert_eq!(tree.len(), 3);
        assert!(tree.root().is_some());
    }

    #[test]
    fn test_merkle_tree_from_data() {
        let data = vec!["a", "b", "c", "d"];
        let tree = MerkleTree::from_data(&data);

        assert_eq!(tree.len(), 4);
        assert!(tree.root().is_some());
    }

    #[test]
    fn test_merkle_tree_from_hashes() {
        let hashes = vec![Hash::from_string("a"), Hash::from_string("b")];
        let tree = MerkleTree::from_hashes(hashes);

        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn test_merkle_tree_consistency() {
        let mut tree1 = MerkleTree::new();
        tree1.push("a");
        tree1.push("b");

        let mut tree2 = MerkleTree::new();
        tree2.push("a");
        tree2.push("b");

        // Same data should produce same root
        assert_eq!(tree1.root(), tree2.root());
    }

    #[test]
    fn test_merkle_tree_different_data() {
        let mut tree1 = MerkleTree::new();
        tree1.push("a");

        let mut tree2 = MerkleTree::new();
        tree2.push("b");

        // Different data should produce different roots
        assert_ne!(tree1.root(), tree2.root());
    }

    #[test]
    fn test_merkle_proof_generation() {
        let mut tree = MerkleTree::new();
        tree.push("a");
        tree.push("b");
        tree.push("c");
        tree.push("d");

        let proof = tree.generate_proof(0).unwrap();
        assert_eq!(proof.leaf_index, 0);
        assert!(!proof.path.is_empty());
        assert_eq!(proof.root, tree.root().unwrap());
    }

    #[test]
    fn test_merkle_proof_out_of_bounds() {
        let tree = MerkleTree::new();
        let result = tree.generate_proof(0);
        assert!(matches!(result, Err(IntegrityError::IndexOutOfBounds(0))));
    }

    #[test]
    fn test_verify_integrity() {
        let mut tree = MerkleTree::new();
        tree.push("a");
        tree.push("b");
        tree.push("c");
        tree.push("d");

        let leaf_hash = Hash::from_string("a");
        let proof = tree.generate_proof(0).unwrap();

        assert!(verify_integrity(leaf_hash, &proof));
    }

    #[test]
    fn test_verify_integrity_invalid() {
        let mut tree = MerkleTree::new();
        tree.push("a");
        tree.push("b");

        let wrong_hash = Hash::from_string("x");
        let proof = tree.generate_proof(0).unwrap();

        assert!(!verify_integrity(wrong_hash, &proof));
    }

    #[test]
    fn test_integrity_proof_convenience() {
        let mut tree = MerkleTree::new();
        tree.push("a");
        tree.push("b");

        let proof = integrity_proof(&tree, 0).unwrap();
        let leaf_hash = Hash::from_string("a");

        assert!(proof.verify(leaf_hash));
    }

    #[test]
    fn test_proof_all_leaves() {
        let data = vec!["a", "b", "c", "d", "e"];
        let tree = MerkleTree::from_data(&data);

        for (i, item) in data.iter().enumerate() {
            let proof = tree.generate_proof(i).unwrap();
            let hash = Hash::from_string(item);
            assert!(verify_integrity(hash, &proof), "proof failed for index {i}");
        }
    }

    #[test]
    fn test_tamper_evident_log() {
        let mut log = TamperEvidentLog::new();

        let hash1 = log.append("entry1");
        let hash2 = log.append("entry2");

        assert_eq!(log.len(), 2);
        assert_ne!(hash1, hash2);
        assert!(log.root().is_some());

        // Verify entries
        let entry0 = log.get(0).unwrap();
        assert_eq!(entry0, b"entry1");
    }

    #[test]
    fn test_tamper_evident_log_prove_verify() {
        let mut log = TamperEvidentLog::new();
        log.append("entry1");
        log.append("entry2");
        log.append("entry3");

        let proof = log.prove(1).unwrap();
        assert!(log.verify(1, &proof));
        assert!(!log.verify(0, &proof)); // Wrong index
    }

    #[test]
    fn test_tamper_evident_log_serializable() {
        let mut log = TamperEvidentLog::new();

        #[derive(Serialize)]
        struct TestEntry {
            name: String,
            value: i32,
        }

        let entry = TestEntry {
            name: "test".to_string(),
            value: 42,
        };

        let hash = log.append_serializable(&entry).unwrap();
        assert!(!hash.is_zero());
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_hash_value() {
        #[derive(Serialize)]
        struct TestData {
            x: i32,
            y: String,
        }

        let data = TestData {
            x: 1,
            y: "hello".to_string(),
        };

        let hash1 = hash_value(&data).unwrap();
        let hash2 = hash_value(&data).unwrap();

        assert_eq!(hash1, hash2);
        assert!(!hash1.is_zero());
    }

    #[test]
    fn test_merkle_tree_clear() {
        let mut tree = MerkleTree::from_data(&["a", "b"]);
        assert!(!tree.is_empty());

        tree.clear();
        assert!(tree.is_empty());
        assert_eq!(tree.root(), None);
    }

    #[test]
    fn test_hash_display() {
        let hash = Hash::from_string("test");
        let display = format!("{hash}");
        assert_eq!(display, hash.to_hex());
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut tree = MerkleTree::new();
        tree.push("a");
        tree.push("b");

        let json = serde_json::to_string(&tree).unwrap();
        let restored: MerkleTree = serde_json::from_str(&json).unwrap();

        assert_eq!(tree.root(), restored.root());
        assert_eq!(tree.len(), restored.len());
    }
}
