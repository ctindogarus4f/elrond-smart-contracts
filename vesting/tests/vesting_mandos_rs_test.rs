use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();

    blockchain.register_contract_builder("file:output/vesting.wasm", vesting::ContractBuilder);
    blockchain
}

#[test]
fn vesting_rs() {
    elrond_wasm_debug::mandos_rs("mandos/vesting.scen.json", world());
}
