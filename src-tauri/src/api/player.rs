use std::collections::HashMap;

use super::local::presence::Player;

pub fn debug_parties(players: &Vec<Player>) {
    let mut parties = HashMap::new();

    for player in players {
        let party = parties
            .entry(player.private.party_id.clone())
            .or_insert(Vec::new());
        party.push(player);
    }

    parties.keys().for_each(|p| {
        debug!("Party: {}", p);
        let party = parties.get(p).unwrap();
        party.iter().for_each(|p| {
            debug!("  {}", p.game_name);
        });
    });
}
