#![no_std]

elrond_wasm::imports!();

/// A staking contract. Users can stake ESDT tokens and gradually receive ESDT token rewards.
#[elrond_wasm::derive::contract]
pub trait StakingContract {
    #[init]
    fn init(&self) {}
}
