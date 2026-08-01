use crate::{Board, Cost, EdgeId, InvalidAction, InvalidBoard, NotEnoughResources, Player, PlayerId, Production, Roll, Scenario};


pub enum GameError {
    BoardInitialization(InvalidBoard),
    Placement(InvalidAction),
    NotEnoughResources,
    NotYourTurn,
    PlayerNotFound(PlayerId),
    GameOver,
}

impl From<InvalidAction> for GameError {
    fn from(e: InvalidAction) -> Self { Self::Placement(e) }
}

impl From<NotEnoughResources> for GameError {
    fn from(_: NotEnoughResources) -> Self { Self::NotEnoughResources }
}

impl From<InvalidBoard> for GameError {
    fn from(e: InvalidBoard) -> Self { Self::BoardInitialization(e) }
}

#[derive(Copy, Clone, Debug)]
pub enum GameStatus {
    Starting,
    Placement,
    Playing,
    End,
}

pub struct Game {
    scenario: Scenario,
    status: GameStatus,
    players: Vec<Player>,
    board: Board,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollOutcome {
    Production(Production),
    RobberActivated { must_discard: Vec<PlayerId> },
}

impl Game {
   

    pub fn apply_roll(&mut self, roll: Roll) -> RollOutcome {
        let outcome = match roll.value() {
            7 => RollOutcome::RobberActivated { must_discard: self.players.iter().enumerate().filter(|(_, p)| p.hand().count() > 7).map(|(i, _)| PlayerId::new(i)).collect::<Vec<PlayerId>>() },
            _ => RollOutcome::Production(self.board.production(roll))
        };

        if let RollOutcome::Production(production) = &outcome {
            production.gains().iter().for_each(|gain| self.players[gain.player.value()].receive(gain.resource, gain.amount));
        }

        outcome
    }

    pub fn build_road(&mut self, player_id: PlayerId, edge: EdgeId) -> Result<(), GameError> {
        self.board.can_place_road(self.status, edge, player_id)?;

        match self.status {
            GameStatus::Placement => {
                self.board.place_road(self.status, edge, player_id)?;
                Ok(())
            }
            GameStatus::Playing => {
                if let Some(player) = self.players.get_mut(player_id.value()) {
                    player.pay(&Cost::ROAD)?;
                    self.board.place_road(self.status, edge, player_id)?;
                    Ok(())
                } else {
                    Err(GameError::PlayerNotFound(player_id))
                }
            }
            _ => Err(GameError::NotYourTurn)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::board::tests::init_board;
    use crate::game::GameStatus::Placement;
    use crate::ResourceCounts;
    use super::*;

    fn init_game() -> Game {
        Game { scenario: Scenario::standard() , status: Placement, players: vec![Player::new(crate::player::PlayerColor::Red), Player::new(crate::player::PlayerColor::Blue)], board: init_board() }
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
        assert_eq!(game.apply_roll(Roll::new(4, 3).unwrap()), RollOutcome::RobberActivated { must_discard: vec![] });
        game.players[0].receive(crate::Resource::Brick, 1);
        assert_eq!(game.apply_roll(Roll::new(4, 3).unwrap()), RollOutcome::RobberActivated { must_discard: vec![PlayerId::new(0)] })
    }

    #[test]
    fn test_build_road() {}
}