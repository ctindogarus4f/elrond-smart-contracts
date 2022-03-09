#[test]
fn init_vesting() {
    elrond_wasm_debug::mandos_go("mandos/init_vesting.scen.json");
}

#[test]
fn group_info_go() {
    elrond_wasm_debug::mandos_go("mandos/group_info.scen.json");
}

#[test]
fn beneficiary_info() {
    elrond_wasm_debug::mandos_go("mandos/beneficiary_info.scen.json");
}

#[test]
fn vesting_logic() {
    elrond_wasm_debug::mandos_go("mandos/vesting_logic.scen.json");
}
