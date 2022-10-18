#![no_std]

elrond_wasm::imports!();

mod types;

use types::*;

const SECONDS_IN_DAY: u64 = 86_400;
const SECONDS_IN_YEAR: u64 = 31_536_000;

/// A staking contract. Users can stake ESDT tokens and gradually receive ESDT token rewards.
#[elrond_wasm::contract]
pub trait StakingContract {
    #[init]
    fn init(&self, token_identifier: TokenIdentifier, total_stake_limit: BigUint) {
        require!(
            token_identifier.is_valid_esdt_identifier(),
            "invalid esdt token"
        );
        self.token_identifier().set_if_empty(&token_identifier);
        self.total_tokens_staked().set_if_empty(&BigUint::zero());
        self.total_stake_limit().set_if_empty(&total_stake_limit);
        self.paused_stake().set_if_empty(&false);
    }

    // endpoints

    #[only_owner]
    #[endpoint(withdrawRewards)]
    fn withdraw_rewards(&self, amount: BigUint) {
        let contract_balance = self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            &self.token_identifier().get(),
            0,
        );
        require!(
            contract_balance >= amount,
            "not enough tokens in the staking contract"
        );

        let caller = self.blockchain().get_caller();
        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &amount,
            b"successful withdraw",
        );

        self.withdraw_rewards_event(&amount);
    }

    #[only_owner]
    #[endpoint(pauseStake)]
    fn pause_stake(&self) {
        self.paused_stake().set(&true);
        self.pause_stake_event();
    }

    #[only_owner]
    #[endpoint(unpauseStake)]
    fn unpause_stake(&self) {
        self.paused_stake().set(&false);
        self.unpause_stake_event();
    }

    #[only_owner]
    #[endpoint(updateStakeLimit)]
    fn update_stake_limit(&self, new_limit: BigUint) {
        self.total_stake_limit().set(&new_limit);
        self.update_stake_limit_event(&new_limit);
    }

    #[only_owner]
    #[endpoint(addPackage)]
    fn add_package(
        &self,
        package_name: ManagedBuffer,
        lock_period: u64,
        apr_percentage: u64,
        rewards_frequency: u64,
        min_stake_amount: BigUint,
    ) {
        require!(
            self.package_info(&package_name).is_empty(),
            "package has already been defined",
        );
        require!(lock_period > 0, "lock period cannot be zero");
        require!(
            apr_percentage > 0 && apr_percentage <= 1_000,
            "apr percentage should be between (0, 1000]"
        );
        require!(
            rewards_frequency <= SECONDS_IN_YEAR,
            "rewards frequency should be less than 365 days"
        );
        require!(
            (lock_period * SECONDS_IN_DAY) % rewards_frequency == 0,
            "the lock period should be divisible with the rewards frequency"
        );

        let package_info = PackageInfo {
            enabled: true,
            lock_period,
            apr_percentage,
            rewards_frequency,
            min_stake_amount,
            total_staked_amount: BigUint::zero(),
        };

        self.package_names().update(|packages| {
            packages.push(package_name.clone());
        });
        self.package_info(&package_name).set(&package_info);
        self.add_package_event(&package_name, &package_info);
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
        self.enable_package_event(&package_name);
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
        self.disable_package_event(&package_name);
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
        let mut staker_ids = self.staker_ids(&caller).get();

        let token_identifier = self.token_identifier().get();
        let (payment_amount, payment_token) = self.call_value().payment_token_pair();
        require!(payment_token == token_identifier, "invalid staked token");
        require!(
            payment_amount >= package_info.min_stake_amount,
            "stake amount too small"
        );

        let total_stake_limit = self.total_stake_limit().get();
        let total_tokens_staked = self.total_tokens_staked().get();
        let new_total_tokens_staked = &total_tokens_staked + &payment_amount;
        require!(
            new_total_tokens_staked <= total_stake_limit,
            "stake limit exceeded"
        );

        self.package_info(&package_name).update(|package| {
            package.total_staked_amount += &payment_amount;
        });
        self.total_tokens_staked().set(&new_total_tokens_staked);

        let staker_info = StakerInfo {
            package_name: package_name.clone(),
            stake_timestamp: self.blockchain().get_block_timestamp(),
            locked_until: self.blockchain().get_block_timestamp()
                + package_info.lock_period * SECONDS_IN_DAY,
            tokens_staked: payment_amount.clone(),
            last_claim_of_rewards: self.blockchain().get_block_timestamp(),
        };

        let staker_counter = self.get_and_increase_staker_counter();
        staker_ids.push(staker_counter);
        self.staker_ids(&caller).set(&staker_ids);
        self.staker_info(staker_counter).set(&staker_info);
        self.create_new_stake_event(
            &caller,
            staker_counter,
            self.blockchain().get_block_timestamp(),
            &package_name,
            &payment_amount,
        );
    }

    #[endpoint(compoundRewardsToExistingStake)]
    fn compound_rewards_to_existing_stake(&self, id: u64) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.staker_ids(&caller).is_empty(),
            "staker does not exist"
        );

        let staker_ids = self.staker_ids(&caller).get();
        require!(staker_ids.contains(&id), "id is not defined for the staker");

        let staker_info = self.staker_info(id).get();
        let package_info = self.package_info(&staker_info.package_name).get();

        let claimable_rewards = self.compute_claimable_rewards(
            &staker_info.tokens_staked,
            package_info.apr_percentage,
            package_info.rewards_frequency,
            staker_info.last_claim_of_rewards,
            staker_info.locked_until,
        );
        require!(claimable_rewards > 0, "no rewards to be claimed");

        self.staker_info(id).update(|staker| {
            staker.tokens_staked += &claimable_rewards;
            staker.last_claim_of_rewards = self.blockchain().get_block_timestamp();
        });
        self.compound_rewards_to_existing_stake_event(
            id,
            self.blockchain().get_block_timestamp(),
            &claimable_rewards,
        );
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

        let claimable_rewards = self.compute_claimable_rewards(
            &staker_info.tokens_staked,
            package_info.apr_percentage,
            package_info.rewards_frequency,
            staker_info.last_claim_of_rewards,
            staker_info.locked_until,
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

        self.staker_info(id).update(|info| {
            info.last_claim_of_rewards = self.blockchain().get_block_timestamp();
        });

        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &claimable_rewards,
            b"successful claim",
        );
        self.claim_rewards_event(
            id,
            self.blockchain().get_block_timestamp(),
            &claimable_rewards,
        );
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

        require!(
            self.blockchain().get_block_timestamp() > staker_info.locked_until,
            "tokens are under locking period"
        );

        let claimable_rewards = self.compute_claimable_rewards(
            &staker_info.tokens_staked,
            package_info.apr_percentage,
            package_info.rewards_frequency,
            staker_info.last_claim_of_rewards,
            staker_info.locked_until,
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

        let index = staker_ids.iter().position(|elem| elem == id).unwrap();
        staker_ids.remove(index);
        if staker_ids.is_empty() {
            self.staker_ids(&caller).clear();
        } else {
            self.staker_ids(&caller).set(&staker_ids);
        }
        self.staker_info(id).clear();

        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &unstake_amount,
            b"successful unstake",
        );
        self.unstake_event(id, self.blockchain().get_block_timestamp(), &unstake_amount);
    }

    #[view(getAvailableRewards)]
    fn get_available_rewards(&self, id: u64) -> BigUint {
        require!(!self.staker_info(id).is_empty(), "stake does not exist");

        let staker_info = self.staker_info(id).get();
        let package_info = self.package_info(&staker_info.package_name).get();

        self.compute_claimable_rewards(
            &staker_info.tokens_staked,
            package_info.apr_percentage,
            package_info.rewards_frequency,
            staker_info.last_claim_of_rewards,
            staker_info.locked_until,
        )
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
        apr_percentage: u64,
        rewards_frequency: u64,
        last_claim: u64,
        locked_until: u64,
    ) -> BigUint {
        let rewards_per_cycle: BigUint =
            self.compute_rewards_per_cycle(staked_amount, apr_percentage, rewards_frequency);
        let cycles_since_last_claim =
            self.compute_cycles_since_last_claim(rewards_frequency, last_claim, locked_until);
        rewards_per_cycle * cycles_since_last_claim
    }

    fn compute_rewards_per_cycle(
        &self,
        staked_amount: &BigUint,
        apr_percentage: u64,
        rewards_frequency: u64,
    ) -> BigUint {
        let cycles_in_one_year = SECONDS_IN_YEAR / rewards_frequency;
        let rewards_per_cycle: BigUint =
            staked_amount * apr_percentage / 100u64 / cycles_in_one_year;
        rewards_per_cycle
    }

    fn compute_cycles_since_last_claim(
        &self,
        rewards_frequency: u64,
        last_claim: u64,
        locked_until: u64,
    ) -> u64 {
        let mut last_eligible_timestamp = self.blockchain().get_block_timestamp();

        if last_eligible_timestamp > locked_until {
            last_eligible_timestamp = locked_until;
        }

        if last_eligible_timestamp <= last_claim {
            return 0;
        }

        let seconds_since_last_claim = last_eligible_timestamp - last_claim;
        seconds_since_last_claim / rewards_frequency
    }

    // events

    #[event("withdraw_rewards")]
    fn withdraw_rewards_event(&self, #[indexed] amount: &BigUint);

    #[event("pause_stake")]
    fn pause_stake_event(&self);

    #[event("unpause_stake")]
    fn unpause_stake_event(&self);

    #[event("update_stake_limit")]
    fn update_stake_limit_event(&self, #[indexed] new_limit: &BigUint);

    #[event("add_package")]
    fn add_package_event(
        &self,
        #[indexed] package_name: &ManagedBuffer,
        #[indexed] package_info: &PackageInfo<Self::Api>,
    );

    #[event("enable_package")]
    fn enable_package_event(&self, #[indexed] package_name: &ManagedBuffer);

    #[event("disable_package")]
    fn disable_package_event(&self, #[indexed] package_name: &ManagedBuffer);

    #[event("create_new_stake")]
    fn create_new_stake_event(
        &self,
        #[indexed] staker: &ManagedAddress,
        #[indexed] stake_id: u64,
        #[indexed] timestamp: u64,
        #[indexed] package_name: &ManagedBuffer,
        #[indexed] amount: &BigUint,
    );

    #[event("compound_rewards_to_existing_stake")]
    fn compound_rewards_to_existing_stake_event(
        &self,
        #[indexed] stake_id: u64,
        #[indexed] timestamp: u64,
        #[indexed] rewards: &BigUint,
    );

    #[event("claim_rewards")]
    fn claim_rewards_event(
        &self,
        #[indexed] stake_id: u64,
        #[indexed] timestamp: u64,
        #[indexed] rewards: &BigUint,
    );

    #[event("unstake")]
    fn unstake_event(
        &self,
        #[indexed] stake_id: u64,
        #[indexed] timestamp: u64,
        #[indexed] amount: &BigUint,
    );

    // storage

    #[view(getTotalTokensStaked)]
    #[storage_mapper("totalTokensStaked")]
    fn total_tokens_staked(&self) -> SingleValueMapper<BigUint>;

    #[view(getTotalStakeLimit)]
    #[storage_mapper("totalStakeLimit")]
    fn total_stake_limit(&self) -> SingleValueMapper<BigUint>;

    #[view(getTokenIdentifier)]
    #[storage_mapper("tokenIdentifier")]
    fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getPausedStake)]
    #[storage_mapper("pausedStake")]
    fn paused_stake(&self) -> SingleValueMapper<bool>;

    #[view(getStakerCounter)]
    #[storage_mapper("stakerCounter")]
    fn staker_counter(&self) -> SingleValueMapper<u64>;

    #[view(getStakerInfo)]
    #[storage_mapper("stakerInfo")]
    fn staker_info(&self, id: u64) -> SingleValueMapper<StakerInfo<Self::Api>>;

    #[view(getStakerIds)]
    #[storage_mapper("stakerIds")]
    fn staker_ids(&self, staker: &ManagedAddress) -> SingleValueMapper<ManagedVec<u64>>;

    #[view(getPackageNames)]
    #[storage_mapper("packageNames")]
    fn package_names(&self) -> SingleValueMapper<ManagedVec<ManagedBuffer>>;

    #[view(getPackageInfo)]
    #[storage_mapper("packageInfo")]
    fn package_info(
        &self,
        package_name: &ManagedBuffer,
    ) -> SingleValueMapper<PackageInfo<Self::Api>>;
}
