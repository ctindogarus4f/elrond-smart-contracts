use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();

    blockchain.register_contract_builder("file:output/staking.wasm", staking::ContractBuilder);
    blockchain
}

#[test]
fn staking_rs() {
    elrond_wasm_debug::mandos_rs("scenarios/staking.scen.json", world());
}
