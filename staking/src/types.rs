elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::elrond_codec::NestedDecodeInput;

#[derive(NestedEncode, NestedDecode, TopEncode, TypeAbi)]
pub struct StakerInfo<M: ManagedTypeApi> {
    pub package_name: ManagedBuffer<M>,
    pub stake_timestamp: u64,
    pub locked_until: u64,
    pub tokens_staked: BigUint<M>,
    pub last_claim_of_rewards: u64,
    pub premature_unstake_timestamp: u64,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TypeAbi)]
pub struct PackageInfo<M: ManagedTypeApi> {
    pub enabled: bool,                   // if enabled, this package accepts stakes
    pub lock_period: u64,                // in days
    pub apr_percentage: u64,             // for 365 days
    pub rewards_frequency: u64,          // in seconds
    pub min_stake_amount: BigUint<M>,    // in tokens
    pub total_staked_amount: BigUint<M>, // tokens staked to this package so far
    pub penalty_seconds: u64, // the number of seconds you need to wait if you prematurely unstake your tokens
    pub penalty_fee: u64, // the fee in % that you need to pay if you prematurely unstake your tokens
}

impl<M: ManagedTypeApi> TopDecode for StakerInfo<M> {
    fn top_decode<I>(input: I) -> Result<Self, DecodeError>
    where
        I: elrond_codec::TopDecodeInput,
    {
        let mut input = input.into_nested_buffer();
        let package_name = ManagedBuffer::dep_decode(&mut input)?;
        let stake_timestamp = u64::dep_decode(&mut input)?;
        let locked_until = u64::dep_decode(&mut input)?;
        let tokens_staked = BigUint::dep_decode(&mut input)?;
        let last_claim_of_rewards = u64::dep_decode(&mut input)?;

        let premature_unstake_timestamp = if input.is_depleted() {
            0
        } else {
            u64::dep_decode(&mut input)?
        };

        Result::Ok(StakerInfo {
            package_name,
            stake_timestamp,
            locked_until,
            tokens_staked,
            last_claim_of_rewards,
            premature_unstake_timestamp,
        })
    }
}

impl<M: ManagedTypeApi> TopDecode for PackageInfo<M> {
    fn top_decode<I>(input: I) -> Result<Self, DecodeError>
    where
        I: elrond_codec::TopDecodeInput,
    {
        let mut input = input.into_nested_buffer();
        let enabled = bool::dep_decode(&mut input)?;
        let lock_period = u64::dep_decode(&mut input)?;
        let apr_percentage = u64::dep_decode(&mut input)?;
        let rewards_frequency = u64::dep_decode(&mut input)?;
        let min_stake_amount = BigUint::dep_decode(&mut input)?;
        let total_staked_amount = BigUint::dep_decode(&mut input)?;

        let [penalty_seconds, penalty_fee] = if input.is_depleted() {
            [0, 0]
        } else {
            [u64::dep_decode(&mut input)?, u64::dep_decode(&mut input)?]
        };

        Result::Ok(PackageInfo {
            enabled,
            lock_period,
            apr_percentage,
            rewards_frequency,
            min_stake_amount,
            total_staked_amount,
            penalty_seconds,
            penalty_fee,
        })
    }
}
