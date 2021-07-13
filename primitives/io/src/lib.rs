//! Jupiter primitives - IO Module

#![allow(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime_interface::runtime_interface;
use sp_std::vec::Vec;

/// Pairing runtime interface
#[runtime_interface]
pub trait Pairing {
    fn wasm() -> bool {
        if let Ok(mode) = ::std::env::var("JIO_MODE") {
            if mode.to_uppercase() == "WASM" {
                return true;
            }
        }

        false
    }

    fn call(_func_id: u32, _input: &[u8]) -> Option<Vec<u8>> {
        // curve::call(func_id, input).ok()
        None
    }

    fn bls12_377_add() {
        // curve::tests::add(0x2a);
    }

    fn bls12_377_mul() {
        // curve::tests::mul(0x2a);
    }

    fn bls12_377_pairing_two() {
        // curve::tests::pairing(0x2a);
    }

    fn bls12_377_pairing_six() {
        // curve::tests::pairing_six(0x2a);
    }

    fn bls12_377_verify() {
        // curve::tests::verify(0x2a);
    }

    fn bls12_381_add() {
        // curve::tests::add(0x2b);
    }

    fn bls12_381_mul() {
        // curve::tests::mul(0x2b);
    }

    fn bls12_381_pairing_two() {
        // curve::tests::pairing(0x2b);
    }

    fn bls12_381_pairing_six() {
        // curve::tests::pairing_six(0x2b);
    }

    fn bls12_381_verify() {
        // curve::tests::verify(0x2b);
    }

    fn bn254_add() {
        // curve::tests::add(0x2c);
    }

    fn bn254_mul() {
        // curve::tests::mul(0x2c);
    }

    fn bn254_pairing_two() {
        // curve::tests::pairing(0x2c);
    }

    fn bn254_pairing_six() {
        // curve::tests::pairing_six(0x2c);
    }

    fn bn254_verify() {
        // curve::tests::verify(0x2c);
    }

    fn bw6_761_add() {
        // curve::tests::add(0x2d);
    }

    fn bw6_761_mul() {
        // curve::tests::mul(0x2d);
    }

    fn bw6_761_pairing_two() {
        // curve::tests::pairing(0x2d);
    }

    fn bw6_761_pairing_six() {
        // curve::tests::pairing_six(0x2d);
    }

    fn bw6_761_verify() {
        // curve::tests::verify(0x2d);
    }
}
