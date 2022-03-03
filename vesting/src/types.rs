use elrond_wasm::{api::ManagedTypeApi, types::BigUint};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct BeneficiaryInfo<M: ManagedTypeApi> {
    pub start: u64,
    pub release_cliff: u64,
    pub release_percentage: u64,
    pub release_duration: u64,
    pub tokens_allocated: BigUint<M>,
    pub tokens_released: BigUint<M>,
}
