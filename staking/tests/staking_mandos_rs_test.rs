use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract("file:output/staking.wasm", staking::ContractBuilder);
    blockchain
}

#[test]
fn staking_rs() {
    multiversx_sc_scenario::run_rs("scenarios/staking.scen.json", world());
}
