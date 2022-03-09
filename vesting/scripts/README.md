Each line inside `groups_data.txt` represents the following:

```bash
<group_name> <group_id> <release_cliff> <release_duration> <release_percentage>
```
where:
- group_name: name of the group (ex: `team`)
- group_id: id of the group (ex: `5`)
- release_cliff: release cliff of the group, measured in seconds (ex: for 12 months cliff we have `60 (seconds) * 60 (minutes) * 24 (hours) * 28 (days) * 12 (months)`=`29,030,400`)
- release_duration: release duration of the group, measured in seconds (ex: for releases that occur 4 months we have `60 (seconds) * 60 (minutes) * 24 (hours) * 28 (days) * 4 (months)`=`9,676,800`)
- release_percentage: release percentage of the group (ex: for 10% we have `10`)
