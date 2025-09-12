use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub struct UTXO {
    pub commitment: [u8; 32],
    pub value: u64,
    pub secret: [u8; 32],
    pub nullifier: [u8; 32],
    pub owner: [u8; 32], // User's public key
    pub index: u64,      // Position in Merkle tree
}

impl UTXO {
    pub fn new(value: u64, secret: [u8; 32], nullifier: [u8; 32], owner: [u8; 32]) -> Self {
        let commitment = Self::compute_commitment(value, &secret, &nullifier);
        Self {
            commitment,
            value,
            secret,
            nullifier,
            owner,
            index: 0, // Will be set when added to tree
        }
    }
    
    pub fn compute_commitment(value: u64, secret: &[u8; 32], nullifier: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&value.to_le_bytes());
        hasher.update(secret);
        hasher.update(nullifier);
        hasher.finalize().into()
    }
    
    pub fn compute_nullifier_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.nullifier);
        hasher.finalize().into()
    }
}

#[derive(Debug, Clone)]
pub struct UTXOInput {
    pub utxo: UTXO,
    pub merkle_proof: MerkleProof,
    pub secret: [u8; 32], // Private key to spend
    pub signature: [u8; 64], // Transaction signature
}

impl serde::Serialize for UTXOInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("UTXOInput", 4)?;
        state.serialize_field("utxo", &self.utxo)?;
        state.serialize_field("merkle_proof", &self.merkle_proof)?;
        state.serialize_field("secret", &self.secret)?;
        state.serialize_field("signature", &self.signature.to_vec())?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for UTXOInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct UTXOInputVisitor;

        impl<'de> Visitor<'de> for UTXOInputVisitor {
            type Value = UTXOInput;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct UTXOInput")
            }

            fn visit_map<V>(self, mut map: V) -> Result<UTXOInput, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut utxo = None;
                let mut merkle_proof = None;
                let mut secret = None;
                let mut signature = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "utxo" => {
                            if utxo.is_some() {
                                return Err(de::Error::duplicate_field("utxo"));
                            }
                            utxo = Some(map.next_value()?);
                        }
                        "merkle_proof" => {
                            if merkle_proof.is_some() {
                                return Err(de::Error::duplicate_field("merkle_proof"));
                            }
                            merkle_proof = Some(map.next_value()?);
                        }
                        "secret" => {
                            if secret.is_some() {
                                return Err(de::Error::duplicate_field("secret"));
                            }
                            secret = Some(map.next_value()?);
                        }
                        "signature" => {
                            if signature.is_some() {
                                return Err(de::Error::duplicate_field("signature"));
                            }
                            let sig_vec: Vec<u8> = map.next_value()?;
                            if sig_vec.len() != 64 {
                                return Err(de::Error::invalid_length(sig_vec.len(), &"64"));
                            }
                            let mut sig_array = [0u8; 64];
                            sig_array.copy_from_slice(&sig_vec);
                            signature = Some(sig_array);
                        }
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                let utxo = utxo.ok_or_else(|| de::Error::missing_field("utxo"))?;
                let merkle_proof = merkle_proof.ok_or_else(|| de::Error::missing_field("merkle_proof"))?;
                let secret = secret.ok_or_else(|| de::Error::missing_field("secret"))?;
                let signature = signature.ok_or_else(|| de::Error::missing_field("signature"))?;

                Ok(UTXOInput {
                    utxo,
                    merkle_proof,
                    secret,
                    signature,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["utxo", "merkle_proof", "secret", "signature"];
        deserializer.deserialize_struct("UTXOInput", FIELDS, UTXOInputVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOOutput {
    pub value: u64,
    pub secret: [u8; 32],
    pub nullifier: [u8; 32],
    pub recipient: [u8; 32],
}

impl UTXOOutput {
    pub fn to_utxo(&self, _index: u64) -> UTXO {
        UTXO::new(self.value, self.secret, self.nullifier, self.recipient)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOTransaction {
    pub inputs: Vec<UTXOInput>,
    pub outputs: Vec<UTXOOutput>,
    pub fee: u64,
    pub tx_type: TransactionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit { amount: u64, recipient: [u8; 32] },
    Withdraw { amount: u64, recipient: [u8; 32] },
    Transfer { recipient: [u8; 32] },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub siblings: Vec<[u8; 32]>,
    pub path: Vec<bool>,
    pub root: [u8; 32],
    pub leaf_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub public_key: [u8; 32],
    pub private_key: [u8; 32],
    pub utxos: Vec<UTXO>,
    pub balance: u64,
}

impl User {
    pub fn new(public_key: [u8; 32], private_key: [u8; 32]) -> Self {
        Self {
            public_key,
            private_key,
            utxos: Vec::new(),
            balance: 0,
        }
    }
    
    pub fn add_utxo(&mut self, utxo: UTXO) {
        let value = utxo.value;
        self.utxos.push(utxo);
        self.balance += value;
    }
    
    pub fn remove_utxo(&mut self, commitment: &[u8; 32]) -> Option<UTXO> {
        if let Some(pos) = self.utxos.iter().position(|u| u.commitment == *commitment) {
            let utxo = self.utxos.remove(pos);
            self.balance -= utxo.value;
            Some(utxo)
        } else {
            None
        }
    }
    
    pub fn can_spend(&self, amount: u64) -> bool {
        self.balance >= amount
    }
    
    pub fn select_utxos_for_spending(&self, amount: u64) -> Vec<&UTXO> {
        let mut selected = Vec::new();
        let mut total = 0u64;
        
        for utxo in &self.utxos {
            selected.push(utxo);
            total += utxo.value;
            if total >= amount {
                break;
            }
        }
        
        selected
    }
    
    // Simple signature verification (in production, use proper crypto)
    pub fn verify_signature(&self, message: &[u8], signature: &[u8; 64], public_key: &[u8; 32]) -> bool {
        // For now, use a simple hash-based verification
        // In production, implement proper ECDSA or Ed25519 verification
        let mut hasher = Sha256::new();
        hasher.update(message);
        hasher.update(public_key);
        let expected_hash = hasher.finalize();
        
        // Simple verification: check if signature starts with expected hash prefix
        signature[..32] == expected_hash[..32]
    }
    
    pub fn sign_transaction(&self, message: &[u8]) -> [u8; 64] {
        // Simple signature generation (in production, use proper crypto)
        let mut hasher = Sha256::new();
        hasher.update(message);
        hasher.update(&self.private_key);
        let hash = hasher.finalize();
        
        let mut signature = [0u8; 64];
        signature[..32].copy_from_slice(&hash);
        signature[32..].copy_from_slice(&self.private_key);
        signature
    }
}
