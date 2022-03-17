#!/bin/bash

if [ -f .env ]; then
  export $(cat .env | xargs)
fi

GROUPS_DATA="../data/groups.txt"

while read group; do
  read name id cliff frequency percentage <<<$group
  # remove thousand separator from numbers
  cliff=$(sed 's/,//g' <<<$cliff)
  frequency=$(sed 's/,//g' <<<$frequency)
  # echo $name $id $cliff $frequency $percentage
  erdpy --verbose contract call --pem $OWNER_WALLET --gas-limit 600000000 --proxy $PROXY --chain $CHAIN --recall-nonce --send $VESTING_SC_ADDRESS --function addGroup --arguments $id $cliff $frequency $percentage
  # erdpy --verbose contract query --proxy $PROXY $VESTING_SC_ADDRESS --function getGroupInfo --arguments $id
done <$GROUPS_DATA
