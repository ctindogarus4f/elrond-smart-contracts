#![no_std]

elrond_wasm::imports!();

mod types;

use types::*;

/// A staking contract. Users can stake ESDT tokens and gradually receive ESDT token rewards.
#[elrond_wasm::contract]
pub trait StakingContract {
    #[init]
    fn init(&self, token_identifier: TokenIdentifier) {
        require!(
            token_identifier.is_valid_esdt_identifier(),
            "invalid esdt token"
        );
        self.token_identifier().set_if_empty(&token_identifier);
        self.total_tokens_allocated().set_if_empty(&BigUint::zero());
        self.paused_stake().set_if_empty(&false);
        self.paused_rewards_timestamp().set_if_empty(&0);
    }

    // endpoints

    #[only_owner]
    #[endpoint(pauseStake)]
    fn pause_stake(&self) {
        self.paused_stake().set(&true);
    }

    #[only_owner]
    #[endpoint(unpauseStake)]
    fn unpause_stake(&self) {
        self.paused_stake().set(&false);
    }

    #[only_owner]
    #[endpoint(pauseRewards)]
    fn pause_rewards(&self) {
        self.paused_rewards_timestamp()
            .set(&self.blockchain().get_block_timestamp());
    }

    #[only_owner]
    #[endpoint(unpauseRewards)]
    fn unpause_rewards(&self) {
        self.paused_rewards_timestamp().set(&0);
    }

    #[only_owner]
    #[endpoint(addPackage)]
    fn add_package(
        &self,
        package_name: ManagedBuffer,
        lock_period: u64,
        apr_percentage: u8,
        rewards_frequency: u64,
        min_stake_amount: BigUint,
    ) {
        require!(
            self.package_info(&package_name).is_empty(),
            "package has already been defined",
        );
        require!(
            apr_percentage > 0 && apr_percentage <= 100,
            "apr percentage should be between (0, 100]"
        );

        let package_info = PackageInfo {
            enabled: true,
            lock_period,
            apr_percentage,
            rewards_frequency,
            min_stake_amount,
            total_staked_amunt: BigUint::zero(),
        };

        self.package_info(&package_name).set(&package_info);
    }

    #[only_owner]
    #[endpoint(enablePackage)]
    fn enable_package(&self, package_name: ManagedBuffer) {
        require!(
            !self.package_info(&package_name).is_empty(),
            "specified package is not set up",
        );
        self.package_info(&package_name).update(|package| {
            package.enabled = true;
        });
    }

    #[only_owner]
    #[endpoint(disablePackage)]
    fn disable_package(&self, package_name: ManagedBuffer) {
        require!(
            !self.package_info(&package_name).is_empty(),
            "specified package is not set up",
        );
        self.package_info(&package_name).update(|package| {
            package.enabled = false;
        });
    }

    #[payable("*")]
    #[endpoint(createNewStake)]
    fn create_new_stake(&self, package_name: ManagedBuffer) {
        require!(!self.paused_stake().get(), "the staking is paused",);
        require!(
            !self.package_info(&package_name).is_empty(),
            "specified package is not set up",
        );

        let package_info = self.package_info(&package_name).get();
        require!(package_info.enabled, "this package is disabled",);

        let caller = self.blockchain().get_caller();
        let mut staker_ids;
        if self.staker_ids(&caller).is_empty() {
            staker_ids = ManagedVec::new();
        } else {
            staker_ids = self.staker_ids(&caller).get();
        }

        let (payment_amount, payment_token) = self.call_value().payment_token_pair();
        require!(
            payment_token == self.token_identifier().get(),
            "invalid staked token"
        );
        require!(
            payment_amount >= package_info.min_stake_amount,
            "stake amount too small"
        );

        self.package_info(&package_name).update(|package| {
            package.total_staked_amunt += &payment_amount;
        });
        self.total_tokens_allocated()
            .update(|tokens| *tokens += &payment_amount);

        let staker_info = StakerInfo {
            package_name,
            stake_timestamp: self.blockchain().get_block_timestamp(),
            tokens_staked: payment_amount,
            last_claim_of_rewards: self.blockchain().get_block_timestamp(),
        };

        let staker_counter = self.get_and_increase_staker_counter();
        staker_ids.push(staker_counter);
        self.staker_ids(&caller).set(&staker_ids);
        self.staker_info(staker_counter).set(&staker_info);
    }

    #[endpoint(reinvestRewardsToExistingStake)]
    fn reinvest_rewards_to_existing_stake(&self, id: u64) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.staker_ids(&caller).is_empty(),
            "staker does not exist"
        );

        let staker_ids = self.staker_ids(&caller).get();
        require!(staker_ids.contains(&id), "id is not defined for the staker");

        let staker_info = self.staker_info(id).get();
        let package_info = self.package_info(&staker_info.package_name).get();

        let locked_until = staker_info.stake_timestamp + package_info.lock_period * 86400;
        let claimable_rewards = self.compute_claimable_rewards(
            &staker_info.tokens_staked,
            package_info.apr_percentage,
            package_info.rewards_frequency,
            staker_info.last_claim_of_rewards,
            locked_until,
        );
        require!(claimable_rewards > 0, "no rewards to be claimed");

        self.staker_info(id).update(|staker| {
            staker.tokens_staked += claimable_rewards;
            staker.last_claim_of_rewards = self.blockchain().get_block_timestamp();
        });
    }

    #[endpoint(claimRewards)]
    fn claim_rewards(&self, id: u64) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.staker_ids(&caller).is_empty(),
            "staker does not exist"
        );

        let staker_ids = self.staker_ids(&caller).get();
        require!(staker_ids.contains(&id), "id is not defined for the staker");

        let staker_info = self.staker_info(id).get();
        let package_info = self.package_info(&staker_info.package_name).get();

        let locked_until = staker_info.stake_timestamp + package_info.lock_period * 86400;
        let claimable_rewards = self.compute_claimable_rewards(
            &staker_info.tokens_staked,
            package_info.apr_percentage,
            package_info.rewards_frequency,
            staker_info.last_claim_of_rewards,
            locked_until,
        );
        require!(claimable_rewards > 0, "no rewards to be claimed");

        let contract_balance = self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            &self.token_identifier().get(),
            0,
        );
        require!(
            contract_balance >= claimable_rewards,
            "not enough tokens in the staking contract"
        );

        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &claimable_rewards,
            b"successful claim",
        );

        self.staker_info(id).update(|info| {
            info.last_claim_of_rewards = self.blockchain().get_block_timestamp();
        });
    }

    #[endpoint]
    fn unstake(&self, id: u64) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.staker_ids(&caller).is_empty(),
            "staker does not exist"
        );

        let mut staker_ids = self.staker_ids(&caller).get();
        require!(staker_ids.contains(&id), "id is not defined for the staker");

        let staker_info = self.staker_info(id).get();
        let package_info = self.package_info(&staker_info.package_name).get();

        let locked_until = staker_info.stake_timestamp + package_info.lock_period * 86400;
        require!(
            self.blockchain().get_block_timestamp() > locked_until,
            "tokens are under locking period"
        );

        let claimable_rewards = self.compute_claimable_rewards(
            &staker_info.tokens_staked,
            package_info.apr_percentage,
            package_info.rewards_frequency,
            staker_info.last_claim_of_rewards,
            locked_until,
        );
        let unstake_amount = staker_info.tokens_staked + claimable_rewards;

        let contract_balance = self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            &self.token_identifier().get(),
            0,
        );
        require!(
            contract_balance >= unstake_amount,
            "not enough tokens in the staking contract"
        );

        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &unstake_amount,
            b"successful unstake",
        );

        let index = staker_ids.iter().position(|elem| elem == id).unwrap();
        staker_ids.remove(index);
        if staker_ids.is_empty() {
            self.staker_ids(&caller).clear();
        } else {
            self.staker_ids(&caller).set(&staker_ids);
        }
        self.staker_info(id).clear();
    }

    #[endpoint(getAvailableRewards)]
    fn get_available_rewards(&self, id: u64) -> BigUint {
        require!(!self.staker_info(id).is_empty(), "stake does not exist");

        let staker_info = self.staker_info(id).get();
        let package_info = self.package_info(&staker_info.package_name).get();

        let locked_until = staker_info.stake_timestamp + package_info.lock_period * 86400;
        let claimable_rewards = self.compute_claimable_rewards(
            &staker_info.tokens_staked,
            package_info.apr_percentage,
            package_info.rewards_frequency,
            staker_info.last_claim_of_rewards,
            locked_until,
        );

        claimable_rewards
    }

    // private functions

    fn get_and_increase_staker_counter(&self) -> u64 {
        let id = self.staker_counter().get();
        self.staker_counter().set(&(id + 1));
        id
    }

    fn compute_claimable_rewards(
        &self,
        staked_amount: &BigUint,
        apr_percentage: u8,
        rewards_frequency: u64,
        last_claim: u64,
        locked_until: u64,
    ) -> BigUint {
        let rewards_per_cycle: BigUint =
            self.compute_rewards_per_cycle(staked_amount, apr_percentage, rewards_frequency);
        let cycles_since_last_claim =
            self.compute_cycles_since_last_claim(rewards_frequency, last_claim, locked_until);
        let claimable_rewards = rewards_per_cycle * cycles_since_last_claim;
        claimable_rewards
    }

    fn compute_rewards_per_cycle(
        &self,
        staked_amount: &BigUint,
        apr_percentage: u8,
        rewards_frequency: u64,
    ) -> BigUint {
        let cycles_in_one_year = 365 / rewards_frequency;
        let rewards_per_cycle: BigUint =
            staked_amount * apr_percentage as u64 / 100u64 / cycles_in_one_year;
        rewards_per_cycle
    }

    fn compute_cycles_since_last_claim(
        &self,
        rewards_frequency: u64,
        last_claim: u64,
        locked_until: u64,
    ) -> u64 {
        let mut last_eligible_timestamp = self.blockchain().get_block_timestamp();

        let paused_rewards_timestamp = self.paused_rewards_timestamp().get();
        if paused_rewards_timestamp != 0 {
            last_eligible_timestamp = paused_rewards_timestamp;
        }

        if last_eligible_timestamp > locked_until {
            last_eligible_timestamp = locked_until;
        }

        if last_eligible_timestamp < last_claim {
            return 0;
        }

        let days_since_last_claim = (last_eligible_timestamp - last_claim) / 86400;
        let cycles_since_last_claim = days_since_last_claim / rewards_frequency;
        cycles_since_last_claim
    }

    // storage

    #[view(getTotalTokensAllocated)]
    #[storage_mapper("totalTokensAllocated")]
    fn total_tokens_allocated(&self) -> SingleValueMapper<BigUint>;

    #[view(getTokenIdentifier)]
    #[storage_mapper("tokenIdentifier")]
    fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getPausedStake)]
    #[storage_mapper("pausedStake")]
    fn paused_stake(&self) -> SingleValueMapper<bool>;

    #[view(getPausedRewardsTimestamp)]
    #[storage_mapper("pausedRewardsTimestamp")]
    fn paused_rewards_timestamp(&self) -> SingleValueMapper<u64>;

    #[view(getStakerCounter)]
    #[storage_mapper("stakerCounter")]
    fn staker_counter(&self) -> SingleValueMapper<u64>;

    #[view(getStakerInfo)]
    #[storage_mapper("stakerInfo")]
    fn staker_info(&self, id: u64) -> SingleValueMapper<StakerInfo<Self::Api>>;

    #[view(getStakerIds)]
    #[storage_mapper("stakerIds")]
    fn staker_ids(&self, staker: &ManagedAddress) -> SingleValueMapper<ManagedVec<u64>>;

    #[view(getPackageInfo)]
    #[storage_mapper("packageInfo")]
    fn package_info(
        &self,
        package_name: &ManagedBuffer,
    ) -> SingleValueMapper<PackageInfo<Self::Api>>;
}
