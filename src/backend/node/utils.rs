use sp_core::storage::StorageKey;
use sp_core::{blake2_128, twox_128};
use subspace_core_primitives::PublicKey;

pub(super) fn account_storage_key(public_key: &PublicKey) -> StorageKey {
    let mut storage_key = Vec::new();

    storage_key.extend_from_slice(&twox_128(b"System"));
    storage_key.extend_from_slice(&twox_128(b"Account"));
    // Next two lines are "blake2_128_concat"
    storage_key.extend_from_slice(&blake2_128(public_key.as_ref()));
    storage_key.extend_from_slice(public_key.as_ref());

    StorageKey(storage_key)
}
