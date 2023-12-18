#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopDecode, TopEncode, TypeAbi)]
pub struct StakerInfo<M: ManagedTypeApi> {
    pub package_name: ManagedBuffer<M>,
    pub stake_timestamp: u64,
    pub locked_until: u64,
    pub tokens_staked: BigUint<M>,
    pub last_claim_of_rewards: u64,
    pub premature_unstake_timestamp: u64,
}

mod staking_proxy {
    multiversx_sc::imports!();
    use crate::StakerInfo;

    #[multiversx_sc::proxy]
    pub trait StakingContract {
        #[view(getStakerInfo)]
        fn staker_info(&self, id: u64) -> StakerInfo<Self::Api>;

        #[view(getStakerIds)]
        fn staker_ids(&self, staker: ManagedAddress) -> ManagedVec<u64>;
    }
}

#[multiversx_sc::contract]
pub trait Dao {
    #[init]
    fn init(&self, staking_contract: ManagedAddress) {
        self.staking_contract().set_if_empty(&staking_contract);
    }

    #[proxy]
    fn staking_contract_proxy(&self, sc_address: ManagedAddress)
        -> staking_proxy::Proxy<Self::Api>;

    #[view(getDaoVoteWeight)]
    fn get_dao_vote_weight_view(
        &self,
        address: ManagedAddress,
        _token: OptionalValue<TokenIdentifier>,
    ) -> BigUint {
        let staking_contract = self.staking_contract().get();
        let staker_ids = self
            .staking_contract_proxy(staking_contract.clone())
            .staker_ids(address)
            .with_gas_limit(60_000_000)
            .execute_on_dest_context::<ManagedVec<u64>>();

        let mut sum: BigUint = BigUint::zero();
        for id in staker_ids.iter() {
            let staker_info = self
                .staking_contract_proxy(staking_contract.clone())
                .staker_info(id)
                .with_gas_limit(60_000_000)
                .execute_on_dest_context::<StakerInfo<Self::Api>>();

            sum += staker_info.tokens_staked;
        }

        sum
    }

    #[view(getStakingContract)]
    #[storage_mapper("stakingContract")]
    fn staking_contract(&self) -> SingleValueMapper<ManagedAddress>;
}
