////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    staking
    (
        addPackage
        claimRewards
        createNewStake
        getAvailableRewards
        getPackageInfo
        getPausedRewardsTimestamp
        getPausedStake
        getStakerCounter
        getStakerIds
        getStakerInfo
        getTokenIdentifier
        getTotalTokensAllocated
        pauseRewards
        pauseStake
        reinvestRewardsToExistingStake
        unpauseRewards
        unpauseStake
        unstake
    )
}

elrond_wasm_node::wasm_empty_callback! {}
