use elrond_wasm::{api::ManagedTypeApi, types::BigUint, types::ManagedBuffer};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct StakerInfo<M: ManagedTypeApi> {
    pub package_name: ManagedBuffer<M>,
    pub start: u64,
    pub tokens_staked: BigUint<M>,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct PackageInfo {
    pub apr_percentage: u8,
    pub locking_period: u64,
}
