pub(crate) const GAMMA: &str = "gamma";
pub(crate) const BETA: &str = "beta";
pub(crate) const ALPHA: &str = "alpha";
pub(crate) const ZETA: &str = "zeta";
pub(crate) const U: &str = "u";

mod converter;
mod hash_to_field;
mod kzg;
mod proof;
mod transcript;
mod verify;

pub(crate) mod error;

use bn::Fr;
pub(crate) use converter::{load_plonk_proof_from_bytes, load_plonk_verifying_key_from_bytes};
pub(crate) use proof::PlonkProof;
pub(crate) use verify::verify_plonk_algebraic;

use error::PlonkError;
use sha2::{Digest, Sha256};

use crate::{decode_sp1_vkey_hash, hash_public_inputs};
/// A verifier for Plonk zero-knowledge proofs.
#[derive(Debug)]
pub struct PlonkVerifier;

impl PlonkVerifier {
    /// # Arguments
    ///
    /// * `proof` - The proof bytes.
    /// * `public_inputs` - The SP1 public inputs.
    /// * `sp1_vkey_hash` - The SP1 vkey hash.
    ///   This is generated in the following manner:
    ///
    /// ```ignore
    /// use sp1_sdk::ProverClient;
    /// let client = ProverClient::new();
    /// let (pk, vk) = client.setup(ELF);
    /// let sp1_vkey_hash = vk.bytes32();
    /// ```
    /// * `plonk_vk` - The Plonk verifying key bytes.
    ///   Usually this will be the [`crate::PLONK_VK_BYTES`] constant.
    ///
    /// # Returns
    ///
    /// A `Result` containing a boolean indicating whether the proof is valid,
    /// or a [`PlonkError`] if verification fails.
    pub fn verify(
        proof: &[u8],
        sp1_public_inputs: &[u8],
        sp1_vkey_hash: &str,
        plonk_vk: &[u8],
    ) -> Result<(), PlonkError> {
        // Hash the vk and get the first 4 bytes.
        let plonk_vk_hash: [u8; 4] = Sha256::digest(plonk_vk)[..4].try_into().unwrap();

        // Check to make sure that this proof was generated by the plonk proving key corresponding to
        // the given plonk vk.
        //
        // SP1 prepends the raw Plonk proof with the first 4 bytes of the plonk vkey to
        // facilitate this check.
        if plonk_vk_hash != proof[..4] {
            return Err(PlonkError::PlonkVkeyHashMismatch);
        }

        let sp1_vkey_hash = decode_sp1_vkey_hash(sp1_vkey_hash)?;

        Self::verify_bytes(
            &proof[4..],
            &sp1_vkey_hash,
            &hash_public_inputs(sp1_public_inputs),
            plonk_vk,
        )
    }

    /// Verifies a PLONK proof using raw byte inputs.
    ///
    /// This is a lower-level verification method that works directly with raw bytes rather than
    /// SP1 native inputs. It's used internally by [`verify`] but can also be called directly
    /// if you already have the required byte arrays.
    ///
    /// # Arguments
    ///
    /// * `proof` - The raw PLONK proof bytes (without the 4-byte vkey hash prefix)
    /// * `sp1_vkey_hash` - The 32-byte SP1 verification key hash
    /// * `public_inputs_hash` - The 32-byte hash of the public inputs
    /// * `plonk_vk` - The PLONK verifying key bytes
    ///
    /// # Returns
    ///
    /// A [`Result`] containing unit `()` if the proof is valid,
    /// or a [`PlonkError`] if verification fails.
    pub fn verify_bytes(
        proof: &[u8],
        sp1_vkey_hash: &[u8; 32],
        public_inputs_hash: &[u8; 32],
        plonk_vk: &[u8],
    ) -> Result<(), PlonkError> {
        let proof = load_plonk_proof_from_bytes(proof).unwrap();
        let plonk_vk = load_plonk_verifying_key_from_bytes(plonk_vk).unwrap();

        let public_inputs = Fr::from_slice(public_inputs_hash).unwrap();
        let sp1_vkey_hash = Fr::from_slice(sp1_vkey_hash).unwrap();
        verify_plonk_algebraic(&plonk_vk, &proof, &[sp1_vkey_hash, public_inputs])
    }
}
