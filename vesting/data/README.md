# groups.txt

Each line represents the following:

```bash
<group_name> <group_id> <release_cliff> <release_frequency> <release_percentage>
```
where:
- `group_name`: name of the group (ex: `team`)
- `group_id`: id of the group (ex: `7`)
- `max_allocation`: max amount of tokens that can be allocated to this group (ex: `84,000,000`)
- `release_cliff`: release cliff of the group, measured in seconds (ex: for 12 months cliff we have `60 (seconds) * 60 (minutes) * 24 (hours) * 30 (days) * 12 (months)`=`31,104,400`)
- `release_frequency`: release frequency of the group, measured in seconds (ex: for releases that occur every 4 months we have `60 (seconds) * 60 (minutes) * 24 (hours) * 30 (days) * 4 (months)`=`10,368,000`)
- `release_percentage`: release percentage of the group (ex: for 10% we have `10`)

# beneficiaries.txt

Each line represents the following:

```bash
<beneficiary_address> <beneficiary_status> <beneficiary_group_id> <beneficiary_start> <beneficiary_tokens_allocated>
```
where:
- `beneficiary_address`: Elrond address of the beneficiary. Needs to be a non-custodial wallet. (ex: `erd1vvsv9kx057g4yukv2k6qytf3yp83dc2v92ztcn3ul45sccsashdqw6dqry`)
- `beneficiary_status`: status of the beneficiary. Can be `permanent` or `temporary`. If `permanent`, beneficiary can no longer be removed from the vesting scheme. If `temporary`, beneficiary can be removed from the vesting scheme if the board members agree.
- `beneficiary_group_id`: group id of the beneficiary (ex: for team we have group id `7`)
- `beneficiary_start`: start date of the vesting scheme, represented as a unix timestamp, in seconds. (ex: for Fri Jul 01 2022 00:00:00 GMT+0000 we have `1,656,633,600`)
- `beneficiary_tokens_allocated`: tokens allocated for the beneficiary (ex: `6,000,000`)
