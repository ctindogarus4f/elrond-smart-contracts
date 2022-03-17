use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();
    blockchain.register_contract_builder("file:output/vesting.wasm", vesting::ContractBuilder);
    blockchain
}

#[test]
fn setup() {
    elrond_wasm_debug::mandos_rs("mandos/setup.scen.json", world());
}

#[test]
fn group_info() {
    elrond_wasm_debug::mandos_rs("mandos/group_info.scen.json", world());
}

#[test]
fn beneficiary_info() {
    elrond_wasm_debug::mandos_rs("mandos/beneficiary_info.scen.json", world());
}

#[test]
fn vesting_logic() {
    elrond_wasm_debug::mandos_rs("mandos/vesting_logic.scen.json", world());
}
