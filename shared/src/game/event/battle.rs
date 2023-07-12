use crate::direction::Direction;
use crate::unit;
use crate::unit::UnitId;

enum StationaryBattleOutcome {}

fn stationary_battle(
    attacker: unit::Model,
    attacking_from: Direction,
    possible_defenders: Vec<(UnitId, unit::Model)>,
) -> Result<(), String> {
    let defender = choose_defender(attacking_from, possible_defenders)?;

    Ok(())
}

fn choose_defender(
    attacking_from: Direction,
    possible_defenders: Vec<(UnitId, unit::Model)>,
) -> Result<(UnitId, unit::Model), String> {
    let (first_defender, remaining_defenders) = match possible_defenders.split_first() {
        None => return Err("no possible defenders".to_string()),
        Some(d) => d,
    };

    let mut ret: (UnitId, unit::Model) = first_defender.clone();

    for (defender_id, defender) in remaining_defenders {}

    Ok(ret)
}
