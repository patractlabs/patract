//! # Randomness Module
//!
//! The Randomness Collect Module provide a random which collect from relay chain.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::sp_runtime::{
    offchain::{http, storage::StorageValueRef, Duration},
    traits::{Hash, Member},
    transaction_validity::{
        InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity,
        ValidTransaction,
    },
    RuntimeAppPublic, RuntimeDebug,
};
use frame_support::{
    decl_module, decl_storage,
    dispatch::DispatchResult,
    traits::{Get, Randomness as RandomnessT},
    Parameter,
};

use sp_consensus_babe::{Epoch, VRF_OUTPUT_LENGTH};
use sp_consensus_vrf::schnorrkel;

use frame_system::{
    ensure_none, ensure_root,
    offchain::{SendTransactionTypes, SubmitTransaction},
};

use codec::{Decode, Encode};
use hex;
use lite_json::{JsonValue, Serialize};
use sp_core::offchain::KeyTypeId;
use sp_std::prelude::*;

pub const RAND_COLL: KeyTypeId = KeyTypeId(*b"rand");

pub mod sr25519 {
    mod app_sr25519 {
        use crate::RAND_COLL;
        use sp_application_crypto::{app_crypto, sr25519};
        app_crypto!(sr25519, RAND_COLL);
    }

    sp_application_crypto::with_pair! {
        /// An randomness_collect keypair using sr25519 as its crypto.
        pub type AuthorityPair = app_sr25519::Pair;
    }

    /// An randomness_collect signature using sr25519 as its crypto.
    pub type AuthoritySignature = app_sr25519::Signature;

    /// An randomness_collect identifier using sr25519 as its crypto.
    pub type AuthorityId = app_sr25519::Public;
}

pub mod ed25519 {
    mod app_ed25519 {
        use crate::RAND_COLL;
        use sp_application_crypto::{app_crypto, ed25519};
        app_crypto!(ed25519, RAND_COLL);
    }

    sp_application_crypto::with_pair! {
        /// An i'm online keypair using ed25519 as its crypto.
        pub type AuthorityPair = app_ed25519::Pair;
    }

    /// An i'm online signature using ed25519 as its crypto.
    pub type AuthoritySignature = app_ed25519::Signature;

    /// An i'm online identifier using ed25519 as its crypto.
    pub type AuthorityId = app_ed25519::Public;
}

pub const OCW_DB_RANDOM: &[u8] = b"ocw_collect_random";

/// The rpc port for fetching randomness.
#[derive(Encode, Decode, Clone)]
pub struct RpcPort(pub Vec<u8>);

#[derive(Encode, Decode, PartialEq, RuntimeDebug, Clone)]
pub struct EpochRecord {
    current_epoch: BabeRandomness,
    next_epoch: BabeRandomness,
}

#[derive(RuntimeDebug, PartialEq, Encode, Decode, Default, Clone)]
pub struct BabeRandomness {
    pub epoch: u64,
    pub start_slot: u64,
    pub duration: u64,
    pub randomness: schnorrkel::Randomness,
}

pub trait Config: SendTransactionTypes<Call<Self>> + frame_system::Config {
    type AuthorityId: Member + Parameter + RuntimeAppPublic + Default + Ord;

    type UnsignedPriority: Get<TransactionPriority>;
}

decl_storage! {
    trait Store for Module<T: Config> as RandomnessCollect {
        /// Who can send unsigned extrinsic to set randomness
        Keys get(fn keys): Vec<T::AuthorityId>;
        /// The `Babe::randomness` from relay chain.
        pub HistoricalRandomness get(fn historical_randomness): map hasher(twox_64_concat) u64 => schnorrkel::Randomness;
        /// The `epoch_index` from relay chain.
        RelayChainEpochIndex get(fn relay_chain_epoch_index): u64;

        pub CurrentEpoch get(fn current_epoch): BabeRandomness;
        pub NextEpoch get(fn next_epoch): BabeRandomness;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        #[weight = 0]
        fn set_key(origin, key: T::AuthorityId) -> DispatchResult {
            ensure_root(origin)?;

            <Keys<T>>::mutate(|keys| {
                keys.push(key);
                keys.sort();
            });
            Ok(())
        }

        #[weight = 0]
        fn set_randomness(
            origin,
            epoch_record: EpochRecord,
            _signature: <T::AuthorityId as RuntimeAppPublic>::Signature,
            _index: u32,
        ) -> DispatchResult {
            ensure_none(origin)?;

            let current_epoch_index = epoch_record.current_epoch.epoch;
            let next_epoch_index = epoch_record.next_epoch.epoch;

            RelayChainEpochIndex::put(current_epoch_index);
            if !HistoricalRandomness::contains_key(current_epoch_index) {
                HistoricalRandomness::insert(current_epoch_index, epoch_record.current_epoch.randomness.clone());
            }
            HistoricalRandomness::insert(next_epoch_index, epoch_record.next_epoch.randomness.clone());

            CurrentEpoch::put(epoch_record.current_epoch);
            NextEpoch::put(epoch_record.next_epoch);

            Ok(())
        }

        fn offchain_worker(_block_number: T::BlockNumber) {
            if sp_io::offchain::is_validator() {
                let privileged_keys = Keys::<T>::get();
                let local_keys = T::AuthorityId::all();

                for local_key in local_keys {
                    privileged_keys.binary_search(&local_key)
                    .ok().map(|index| {
                        let signer: T::AuthorityId = local_key.clone();
                        let res = Self::fetch_epoch_and_send_signed(signer, index as u32);
                        if let Err(e) = res {
                            log::error!("Error: {}", e);
                        }
                    });
                }
            } else {
                log::warn!(
                    target: "randomness_collect",
                    "Skipping collect. Not a validator.",
                )
            }
        }
    }
}

impl<T: Config> Module<T> {
    fn get_rpc_port() -> Option<RpcPort> {
        let storage = StorageValueRef::persistent(OCW_DB_RANDOM);
        let rpc_port: Option<Option<RpcPort>> = storage.get();

        match rpc_port {
            Some(Some(rpc_port)) => Some(rpc_port),
            _ => None,
        }
    }

    fn send_post<'a, 'b>(
        url: &'a str,
        post_data: &'a str,
        response_body: &'b mut Vec<u8>,
    ) -> Result<&'b str, http::Error> {
        let request = http::Request::post(url, vec![post_data.as_bytes()]);

        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(1_500));
        let pending = request
            .add_header("Content-Type", "application/json")
            .deadline(deadline)
            .send()
            .map_err(|_| http::Error::IoError)?;

        let response = pending
            .try_wait(deadline)
            .map_err(|_| http::Error::DeadlineReached)??;
        if response.code != 200 {
            log::warn!("Unexpected status code: {}", response.code);
            return Err(http::Error::Unknown);
        }

        *response_body = response.body().collect::<Vec<u8>>();
        sp_std::str::from_utf8(response_body).map_err(|_| {
            log::warn!("No UTF8 body");
            http::Error::Unknown
        })
    }

    fn parse_json_result<'a, 'b>(body: &'a str, buffer: &'b mut Vec<u8>) -> Option<&'b str> {
        let val = lite_json::parse_json(body);

        val.ok().and_then(move |v| match v {
            JsonValue::Object(obj) => obj
                .into_iter()
                .find(|(k, _)| {
                    let mut chars = "result".chars();
                    k.iter().all(|k| Some(*k) == chars.next())
                })
                .and_then(move |v| match v.1 {
                    JsonValue::String(_) => {
                        v.1.serialize_to(buffer, 0, 0);
                        sp_std::str::from_utf8(buffer)
                            .ok()
                            .and_then(|str| Some(str.trim_matches('"').trim_start_matches("0x")))
                    }
                    _ => None,
                }),
            _ => None,
        })
    }

    fn parse_epoch(body: &str) -> Option<Epoch> {
        let mut buffer = Vec::new();
        let epoch = Self::parse_json_result(body, &mut buffer)?;

        hex::decode(epoch)
            .ok()
            .and_then(|epoch_u8| Epoch::decode(&mut epoch_u8.as_slice()).ok())
    }

    fn fetch_epoch(url: &str, method: &str) -> Result<BabeRandomness, http::Error> {
        let mut epoch_response_body = Vec::new();
        let epoch_response = Self::send_post(url, method, &mut epoch_response_body)?;

        let res = match Self::parse_epoch(epoch_response) {
            Some(epoch) => Ok(BabeRandomness {
                epoch: epoch.epoch_index,
                start_slot: *epoch.start_slot,
                duration: epoch.duration,
                randomness: epoch.randomness,
            }),
            None => {
                log::warn!(
                    "Unable to extract epoch from the response: {:?}",
                    epoch_response
                );
                Err(http::Error::Unknown)
            }
        };
        res
    }

    fn fetch_epoch_and_send_signed(signer: T::AuthorityId, index: u32) -> Result<(), &'static str> {
        let rpc_port = Self::get_rpc_port();
        if let Some(rpc_port) = rpc_port {
            match sp_std::str::from_utf8(&[b"http://", rpc_port.0.as_slice()].concat()) {
                Ok(rpc_port_str) => {
                    let post_current_epoch = r#"
			            {
				            "id": "1",
				            "jsonrpc": "2.0",
				            "method": "state_call",
				            "params": ["BabeApi_current_epoch", "0x"]
			            }"#;
                    let post_next_epoch = r#"
			            {
				            "id": "2",
				            "jsonrpc": "2.0",
				            "method": "state_call",
				            "params": ["BabeApi_next_epoch", "0x"]
			            }"#;
                    let current_epoch = Self::fetch_epoch(rpc_port_str, post_current_epoch)
                        .map_err(|_| "Fetch randomness failed")?;
                    let current_epoch_index = Self::relay_chain_epoch_index();
                    if current_epoch.epoch > current_epoch_index {
                        let next_epoch = Self::fetch_epoch(rpc_port_str, post_next_epoch)
                            .map_err(|_| "Fetch randomness failed")?;
                        if next_epoch.epoch == current_epoch.epoch + 1 {
                            let epoch_record = EpochRecord {
                                current_epoch,
                                next_epoch,
                            };
                            let signature = signer
                                .sign(&epoch_record.encode())
                                .ok_or("Failed to sign.")?;
                            let call = Call::set_randomness(epoch_record, signature, index);
                            SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(
                                call.into(),
                            )
                            .map_err(|_| "Failed to submit unsigned transaction")?;
                        } else {
                            Err("Get next epoch wrong")?;
                        }
                    }
                }
                _ => Err("Get rpc_port failed")?,
            }
        }

        Ok(())
    }

    pub fn randomness_of(epoch: u64) -> schnorrkel::Randomness {
        <HistoricalRandomness>::get(&epoch)
    }

    pub fn random(subject: &[u8]) -> (T::Hash, T::BlockNumber) {
        let random = {
            let mut subject = subject.to_vec();
            subject.reserve(VRF_OUTPUT_LENGTH);
            subject.extend_from_slice(
                &Self::historical_randomness(&Self::relay_chain_epoch_index())[..],
            );
            <T as frame_system::Config>::Hashing::hash(&subject[..])
        };

        (random, <frame_system::Pallet<T>>::block_number())
    }
}

impl<T: Config> RandomnessT<T::Hash, T::BlockNumber> for Module<T> {
    fn random(subject: &[u8]) -> (T::Hash, T::BlockNumber) {
        Self::random(subject)
    }
}

#[allow(deprecated)]
impl<T: Config> frame_support::unsigned::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

    fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
        if let Call::set_randomness(ref epoch_record, ref signature, index) = call {
            // check signature
            match Keys::<T>::get().get(*index as usize) {
                None => InvalidTransaction::Call.into(),
                Some(privileged_key) => {
                    let signature_valid = epoch_record.using_encoded(|encoded_epoch_record| {
                        privileged_key.verify(&encoded_epoch_record, &signature)
                    });
                    if !signature_valid {
                        InvalidTransaction::BadProof.into()
                    } else {
                        ValidTransaction::with_tag_prefix("RandomnessCollect")
                            .priority(T::UnsignedPriority::get())
                            .and_provides((epoch_record.current_epoch.epoch, *index))
                            .longevity(5)
                            .propagate(true)
                            .build()
                    }
                }
            }
        } else {
            InvalidTransaction::Call.into()
        }
    }
}
