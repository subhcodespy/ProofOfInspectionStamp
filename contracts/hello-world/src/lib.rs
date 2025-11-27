#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Bytes, Env, String, Symbol,
};

const STAMP_NS: Symbol = symbol_short!("PINSPCT"); // namespace tag

#[contracttype]
#[derive(Clone)]
pub struct InspectionStamp {
    pub stamp_id: u64,        // unique inspection stamp id
    pub asset_id: String,     // thing inspected (machine-id, batch-id, etc.)
    pub inspector: Symbol,    // inspector account symbol
    pub passed: bool,         // pass/fail
    pub notes: String,        // short notes or reason
    pub evidence_hash: Bytes, // optional off-chain report hash
    pub inspected_at: u64,    // timestamp from ledger
    pub revoked: bool,        // mark stamp as no longer valid
}

#[contract]
pub struct ProofOfInspectionStamp;

#[contractimpl]
impl ProofOfInspectionStamp {
    /// Create a new inspection stamp for an asset.
    /// Assumes inspector identity/role is handled by off-chain or outer access control.
    pub fn create_stamp(
        env: Env,
        stamp_id: u64,
        asset_id: String,
        inspector: Symbol,
        passed: bool,
        notes: String,
        evidence_hash: Bytes,
    ) {
        let inspected_at = env.ledger().timestamp();

        let stamp = InspectionStamp {
            stamp_id,
            asset_id,
            inspector,
            passed,
            notes,
            evidence_hash,
            inspected_at,
            revoked: false,
        };

        let key = Self::stamp_key(stamp_id);
        env.storage().instance().set(&key, &stamp);
    }

    /// Revoke an existing inspection stamp (e.g., later defect discovered).
    pub fn revoke_stamp(env: Env, stamp_id: u64) {
        let key = Self::stamp_key(stamp_id);
        let mut stamp: InspectionStamp = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| panic!("Stamp not found"));

        stamp.revoked = true;
        env.storage().instance().set(&key, &stamp);
    }

    /// Check whether a given stamp is currently valid and passed.
    pub fn is_stamp_valid(env: Env, stamp_id: u64) -> bool {
        let key = Self::stamp_key(stamp_id);
        let stamp_opt: Option<InspectionStamp> = env.storage().instance().get(&key);

        match stamp_opt {
            Some(s) => s.passed && !s.revoked,
            None => false,
        }
    }

    /// Get full inspection stamp details by stamp_id.
    pub fn get_stamp(env: Env, stamp_id: u64) -> Option<InspectionStamp> {
        let key = Self::stamp_key(stamp_id);
        env.storage().instance().get(&key)
    }

    /// Helper: derive a composite storage key under a namespace symbol.
    fn stamp_key(stamp_id: u64) -> (Symbol, u64) {
        (STAMP_NS, stamp_id)
    }
}
