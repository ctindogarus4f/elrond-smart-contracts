use elrond_wasm::{api::ManagedTypeApi, types::BigUint, types::ManagedBuffer};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct StakerInfo<M: ManagedTypeApi> {
    pub package_name: ManagedBuffer<M>,
    pub stake_timestamp: u64,
    pub locked_until: u64,
    pub tokens_staked: BigUint<M>,
    pub last_claim_of_rewards: u64,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct PackageInfo<M: ManagedTypeApi> {
    pub enabled: bool,                   // if enabled, this package accepts stakes
    pub lock_period: u64,                // in days
    pub apr_percentage: u64,             // for 365 days
    pub rewards_frequency: u64,          // in seconds
    pub min_stake_amount: BigUint<M>,    // in tokens
    pub total_staked_amount: BigUint<M>, // tokens staked to this package so far
}
