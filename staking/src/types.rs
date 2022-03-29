elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct PackageInfo {
    pub apr_percentage: u8,
    pub locking_period: u32,
}
