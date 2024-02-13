use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    ScenarioWorld::vm_go()
}

#[test]
fn setup() {
    world().run("mandos/setup.scen.json");
}

#[test]
fn group_info() {
    world().run("mandos/group_info.scen.json");
}

#[test]
fn beneficiary_info() {
    world().run("mandos/beneficiary_info.scen.json");
}

#[test]
fn vesting_logic() {
    world().run("mandos/vesting_logic.scen.json");
}
