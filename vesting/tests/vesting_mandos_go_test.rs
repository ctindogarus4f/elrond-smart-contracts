#[test]
fn setup() {
    elrond_wasm_debug::mandos_go("scenarios/setup.scen.json");
}

#[test]
fn group_info() {
    elrond_wasm_debug::mandos_go("scenarios/group_info.scen.json");
}

#[test]
fn beneficiary_info() {
    elrond_wasm_debug::mandos_go("scenarios/beneficiary_info.scen.json");
}

#[test]
fn vesting_logic() {
    elrond_wasm_debug::mandos_go("scenarios/vesting_logic.scen.json");
}
