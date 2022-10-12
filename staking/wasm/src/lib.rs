////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    staking
    (
        addPackage
        claimRewards
        compoundRewardsToExistingStake
        createNewStake
        disablePackage
        enablePackage
        getAvailableRewards
        getPackageInfo
        getPausedStake
        getStakerCounter
        getStakerIds
        getStakerInfo
        getTokenIdentifier
        getTotalStakeLimit
        getTotalTokensStaked
        pauseStake
        unpauseStake
        unstake
        updateStakeLimit
    )
}

elrond_wasm_node::wasm_empty_callback! {}
