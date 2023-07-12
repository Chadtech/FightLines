use crate::direction::Direction;
use crate::unit;
use crate::unit::UnitId;

enum StationaryBattleOutcome {
    Todo,
}

fn stationary_battle(
    _attacker: unit::Model,
    attacking_from: Direction,
    possible_defenders: Vec<(UnitId, unit::Model)>,
) -> Result<StationaryBattleOutcome, String> {
    let defender = choose_defender(attacking_from, possible_defenders)?;

    Ok(StationaryBattleOutcome::Todo)
}

fn choose_defender(
    _attacking_from: Direction,
    possible_defenders: Vec<(UnitId, unit::Model)>,
) -> Result<(UnitId, unit::Model), String> {
    let (first_defender, remaining_defenders) = match possible_defenders.split_first() {
        None => return Err("no possible defenders".to_string()),
        Some(d) => d,
    };

    let ret: (UnitId, unit::Model) = first_defender.clone();

    for (_defender_id, _defender) in remaining_defenders {}

    Ok(ret)
}
