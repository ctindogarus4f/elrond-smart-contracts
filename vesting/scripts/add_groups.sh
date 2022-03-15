#!/bin/bash

if [ -f .env ]; then
  export $(cat .env | xargs)
fi

GROUPS_DATA="groups_data.txt"

while read group; do
  read name id cliff duration percentage <<<$group
  # remove thousand separator from numbers
  cliff=$(sed 's/,//g' <<<$cliff)
  duration=$(sed 's/,//g' <<<$duration)
  # echo $name $id $cliff $duration $percentage
  erdpy --verbose contract call --pem $OWNER_WALLET --gas-limit 600000000 --proxy $PROXY --chain $CHAIN --recall-nonce --send $VESTING_SC_ADDRESS --function addGroup --arguments $id $cliff $duration $percentage
  # erdpy --verbose contract query --proxy $PROXY $VESTING_SC_ADDRESS --function getGroupInfo --arguments $id
done <$GROUPS_DATA
