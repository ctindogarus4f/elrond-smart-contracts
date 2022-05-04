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
    }

    // endpoints

    #[only_owner]
    #[endpoint(addPackageWithPenaltyFee)]
    fn add_package_with_penalty_fee(
        &self,
        package_name: ManagedBuffer,
        apr_percentage: u8,
        duration: u64,
        daily_rewards: bool,
        penalty_fee: u8,
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
            apr_percentage,
            duration,
            daily_rewards,
            penalty_type: PenaltyType::FeePercentage { fee: penalty_fee },
        };

        self.package_info(&package_name).set(&package_info);
    }

    #[only_owner]
    #[endpoint(addPackageWithPenaltyDays)]
    fn add_package_with_penalty_days(
        &self,
        package_name: ManagedBuffer,
        apr_percentage: u8,
        duration: u64,
        daily_rewards: bool,
        penalty_days: u8,
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
            apr_percentage,
            duration,
            daily_rewards,
            penalty_type: PenaltyType::DaysUntilUnlocked { days: penalty_days },
        };

        self.package_info(&package_name).set(&package_info);
    }

    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.staker_info(&caller).is_empty(),
            "staker does not exist",
        );

        let staker_info = self.staker_info(&caller).get();
        let package_info = self.package_info(&staker_info.package_name).get();

        require!(
            package_info.daily_rewards,
            "this package does not offer daily rewards"
        );

        require!(
            staker_info.start + package_info.duration >= self.blockchain().get_block_timestamp(),
            "this package has expired. you need to unstake your tokens"
        );

        let hours_since_last_claim =
            self.blockchain().get_block_timestamp() - staker_info.last_claim / 3600;
        let rewards_per_hour: BigUint = staker_info.tokens_staked
            * package_info.apr_percentage as u64
            / 100u64
            / 365u64
            / 24u64;
        let claimable_rewards = rewards_per_hour * hours_since_last_claim;

        require!(claimable_rewards > 0, "no rewards to be claimed");

        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &claimable_rewards,
            b"successful claim",
        );

        self.staker_info(&caller).update(|info| {
            info.last_claim = self.blockchain().get_block_timestamp();
        });
    }

    #[payable("*")]
    #[endpoint]
    fn stake(&self, package_name: ManagedBuffer) {
        require!(
            !self.package_info(&package_name).is_empty(),
            "specified package is not set up",
        );

        let (payment_amount, payment_token) = self.call_value().payment_token_pair();
        require!(
            payment_token == self.token_identifier().get(),
            "invalid staked token"
        );

        let caller = self.blockchain().get_caller();
        require!(
            self.staker_info(&caller).is_empty(),
            "staker has already been added",
        );

        let package_info = self.package_info(&package_name).get();


        let unstake_amount =
            self.compute_unstake_amount(&payment_amount, package_info.apr_percentage);
        self.total_tokens_allocated()
            .update(|tokens| *tokens += unstake_amount);

        let esdt_balance = self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            &self.token_identifier().get(),
            0,
        );
        require!(
            esdt_balance >= self.total_tokens_allocated().get(),
            "not enough tokens in staking contract"
        );

        let staker_info = StakerInfo {
            last_claim: if package_info.daily_rewards {
                self.blockchain().get_block_timestamp()
            } else {
                0
            },
            package_name,
            start: self.blockchain().get_block_timestamp(),
            tokens_staked: payment_amount,
        };

        self.staker_info(&caller).set(&staker_info);
    }

    #[endpoint]
    fn unstake(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.staker_info(&caller).is_empty(),
            "staker does not exist",
        );

        let staker_info = self.staker_info(&caller).get();
        let package_info = self.package_info(&staker_info.package_name).get();

        require!(
            staker_info.start + package_info.duration >= self.blockchain().get_block_timestamp(),
            "cannot unstake sooner than the locking period"
        );

        let unstake_amount =
            self.compute_unstake_amount(&staker_info.tokens_staked, package_info.apr_percentage);
        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &unstake_amount,
            b"successful unstake",
        );

        self.staker_info(&caller).clear();
    }

    // private functions

    fn compute_unstake_amount(&self, staked_amount: &BigUint, apr_percentage: u8) -> BigUint {
        staked_amount * (100 + apr_percentage) as u64 / 100u64
    }

    // storage

    #[view(getTotalTokensAllocated)]
    #[storage_mapper("totalTokensAllocated")]
    fn total_tokens_allocated(&self) -> SingleValueMapper<BigUint>;

    #[view(getTokenIdentifier)]
    #[storage_mapper("tokenIdentifier")]
    fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getStakerInfo)]
    #[storage_mapper("stakerInfo")]
    fn staker_info(&self, staker: &ManagedAddress) -> SingleValueMapper<StakerInfo<Self::Api>>;

    #[view(getPackageInfo)]
    #[storage_mapper("packageInfo")]
    fn package_info(&self, package_name: &ManagedBuffer) -> SingleValueMapper<PackageInfo>;
}
