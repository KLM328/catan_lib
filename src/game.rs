use crate::{Board, Player, Production, Roll};
use crate::player::PlayerId;

pub struct Game {
    players: Vec<Player>,
    board: Board
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollOutcome {
    Production(Production),
    RobberActivated { must_discard: Vec<PlayerId> },
}

impl Game {
    pub fn apply_roll(&mut self, roll: Roll) -> RollOutcome {
        let outcome = match roll.value() {
            7 => RollOutcome::RobberActivated { must_discard: self.players.iter().enumerate().filter(|(_, p)| p.hand().count() > 7).map(|(i,_)| PlayerId::new(i)).collect::<Vec<PlayerId>>()},
            _ => RollOutcome::Production(self.board.production(roll))
        };

        match &outcome {
            RollOutcome::RobberActivated{must_discard : _ } => {},
            RollOutcome::Production(production) => {
                production.gains().iter().for_each(|gain| self.players[gain.player.id()].receive(gain.resource, gain.amount));
            }

        }

        outcome
    }
}

#[cfg(test)]
mod tests {
    use crate::board::tests::init_board;
    use crate::ResourceCounts;
    use super::*;

    fn init_game() -> Game{
        Game{players : vec![Player::new(crate::player::PlayerColor::Red), Player::new(crate::player::PlayerColor::Blue)], board: init_board()}
    }

    #[test]
    fn test_apply_roll_with_production() {
        let mut game = init_game();
        game.apply_roll(Roll::new(2, 4).unwrap());
        assert_eq!(game.players[1].hand().resources(), ResourceCounts::new([2, 0, 0, 0, 0]));
    }

    #[test]
    fn test_apply_roll_with_robber() {
        let mut game = init_game();
        game.players[0].receive(crate::Resource::Wood, 3);
        game.players[0].receive(crate::Resource::Wheat, 2);
        game.players[0].receive(crate::Resource::Stone, 2);
        assert_eq!(game.apply_roll(Roll::new(4, 3).unwrap()), RollOutcome::RobberActivated { must_discard: vec![]});
        game.players[0].receive(crate::Resource::Brick, 1);
        assert_eq!(game.apply_roll(Roll::new(4, 3).unwrap()), RollOutcome::RobberActivated { must_discard: vec![PlayerId::new(0)]})
    }
}