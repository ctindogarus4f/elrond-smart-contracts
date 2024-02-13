use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract("file:output/vesting.wasm", vesting::ContractBuilder);
    blockchain
}

#[test]
fn setup() {
    world().run("scenarios/setup.scen.json");
}

#[test]
fn group_info() {
    world().run("scenarios/group_info.scen.json");
}

#[test]
fn beneficiary_info() {
    world().run("scenarios/beneficiary_info.scen.json");
}

#[test]
fn vesting_logic() {
    world().run("scenarios/vesting_logic.scen.json");
}
