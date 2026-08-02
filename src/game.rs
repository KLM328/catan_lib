use crate::{
    Board, Cost, EdgeId, InvalidAction, InvalidBoard, NotEnoughResources, Player, PlayerId,
    Production, Roll, Scenario, Terrain, VertexId,
};

#[derive(Debug, PartialEq)]
pub enum GameError {
    BoardInitialization(InvalidBoard),
    Placement(InvalidAction),
    NotEnoughResources,
    NotYourTurn,
    GameOver,
    GameIsStarting,
    GameIsNotPlaying,
    WrongRollCount,
    TiedRolls,
    NotEnoughPlayers,
    PlayerNotFound(PlayerId),
    InvalidPlacementTurn,
}

impl From<InvalidAction> for GameError {
    fn from(e: InvalidAction) -> Self {
        Self::Placement(e)
    }
}

impl From<NotEnoughResources> for GameError {
    fn from(_: NotEnoughResources) -> Self {
        Self::NotEnoughResources
    }
}

impl From<InvalidBoard> for GameError {
    fn from(e: InvalidBoard) -> Self {
        Self::BoardInitialization(e)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum GameStatus {
    Starting,
    Placement(Placement),
    Playing,
    End,
}

#[derive(Clone, Copy, Debug)]
pub struct Placement {
    turn: u8,
}

impl Placement {
    pub fn new(turn: u8) -> Result<Self, GameError> {
        if matches!(turn, 1 | 2) {
            Ok(Self { turn })
        } else {
            Err(GameError::InvalidPlacementTurn)
        }
    }
}

pub struct Game {
    scenario: Scenario,
    status: GameStatus,
    players: Vec<Player>,
    turn_order: Vec<PlayerId>,
    current_turn: usize,
    board: Option<Board>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollOutcome {
    Production(Production),
    RobberActivated { must_discard: Vec<PlayerId> },
}

impl Game {
    pub fn new(scenario: Scenario, players: Vec<Player>) -> Result<Game, GameError> {
        if players.is_empty() {
            Err(GameError::NotEnoughPlayers)
        } else {
            Ok(Game {
                scenario,
                status: GameStatus::Starting,
                turn_order: (0..players.len())
                    .into_iter()
                    .map(|i| PlayerId::new(i))
                    .collect(),
                current_turn: 0,
                players,
                board: None,
            })
        }
    }

    fn board(&self) -> Result<&Board, GameError> {
        self.board.as_ref().ok_or(GameError::GameIsStarting)
    }
    fn board_mut(&mut self) -> Result<&mut Board, GameError> {
        self.board.as_mut().ok_or(GameError::GameIsStarting)
    }
    pub fn start(&mut self, shuffled: &[Terrain]) -> Result<(), GameError> {
        self.board = Some(self.scenario.layout(shuffled)?);
        self.set_status(GameStatus::Placement(Placement::new(1)?));
        Ok(())
    }

    pub(crate) fn set_status(&mut self, status: GameStatus) {
        self.status = status;
    }

    pub fn status(&self) -> GameStatus {
        self.status
    }

    pub fn current_player(&self) -> PlayerId {
        self.turn_order[self.current_turn]
    }

    pub fn next_player(&mut self) -> Result<(), GameError> {
        if let GameStatus::Placement(Placement { turn }) = self.status {
            match turn {
                1 => {
                    if self.current_turn == self.turn_order.len() - 1 {
                        self.set_status(GameStatus::Placement(Placement::new(2)?));
                        Ok(())
                    } else {
                        self.current_turn = self.current_turn + 1;
                        Ok(())
                    }
                }
                2 => {
                    if self.current_turn == 0 {
                        self.set_status(GameStatus::Playing);
                        Ok(())
                    } else {
                        self.current_turn = self.current_turn - 1;
                        Ok(())
                    }
                },
                _ => unreachable!()
            }
        } else {
            self.current_turn = (self.current_turn + 1) % self.turn_order.len();
            Ok(())
        }
    }

    pub fn set_players_order(&mut self, rolls: Vec<Roll>) -> Result<(), GameError> {
        if rolls.len() == self.players.len() {
            let best = rolls.iter().map(|r| r.value()).max().unwrap();
            if rolls.iter().filter(|r| r.value() == best).count() > 1 {
                Err(GameError::TiedRolls)
            } else {
                let first = rolls.iter().position(|r| r.value() == best).unwrap();
                let n = self.players.len();
                self.turn_order = (0..n).map(|k| PlayerId::new((first + k) % n)).collect();
                Ok(())
            }
        } else {
            Err(GameError::WrongRollCount)
        }
    }

    pub fn playable_status(&self) -> Result<(), GameError> {
        match self.status {
            GameStatus::Starting => Err(GameError::GameIsStarting),
            GameStatus::Placement(_) => Ok(()),
            GameStatus::Playing => Ok(()),
            GameStatus::End => Err(GameError::GameOver),
        }
    }

    pub fn apply_roll(&mut self, roll: Roll) -> Result<RollOutcome, GameError> {
        self.playable_status()?;
        let outcome = match roll.value() {
            7 => RollOutcome::RobberActivated {
                must_discard: self
                    .players
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.hand().count() > 7)
                    .map(|(i, _)| PlayerId::new(i))
                    .collect::<Vec<PlayerId>>(),
            },
            _ => RollOutcome::Production(self.board()?.production(roll)),
        };

        if let RollOutcome::Production(production) = &outcome {
            production.gains().iter().for_each(|gain| {
                self.players[gain.player.value()].receive(gain.resource, gain.amount)
            });
        }

        Ok(outcome)
    }

    pub fn build_road(&mut self, player_id: PlayerId, edge: EdgeId) -> Result<(), GameError> {
        self.playable_status()?;
        self.check_player(player_id)?;
        let current_status = self.status;

        self.board
            .as_ref()
            .unwrap()
            .can_place_road(self.status, edge, player_id)?;

        if matches!(self.status, GameStatus::Playing) {
            self.get_player_mut(player_id)?.pay(&Cost::ROAD)?;
        }
        self.board_mut()?
            .place_road(current_status, edge, player_id)?;
        Ok(())
    }

    pub fn build_settlement(
        &mut self,
        player_id: PlayerId,
        vertex: VertexId,
    ) -> Result<(), GameError> {
        self.playable_status()?;
        self.check_player(player_id)?;
        let current_status = self.status;
        self.board()?
            .can_place_building(self.status, vertex, player_id)?;

        if matches!(self.status, GameStatus::Playing) {
            self.get_player_mut(player_id)?.pay(&Cost::SETTLEMENT)?;
        }
        self.board_mut()?
            .place_settlement(current_status, vertex, player_id)?;
        Ok(())
    }

    pub fn check_player(&self, player_id: PlayerId) -> Result<(), GameError> {
        if player_id == self.current_player() {
            Ok(())
        } else {
            Err(GameError::NotYourTurn)
        }
    }
    pub fn get_player_mut(&mut self, player_id: PlayerId) -> Result<&mut Player, GameError> {
        let Some(player) = self.players.get_mut(player_id.value()) else {
            return Err(GameError::PlayerNotFound(player_id));
        };
        Ok(player)
    }

    pub fn upgrade_settlement_to_city(
        &mut self,
        player_id: PlayerId,
        vertex: VertexId,
    ) -> Result<(), GameError> {
        self.playable_status()?;
        self.check_player(player_id)?;
        if matches!(self.status, GameStatus::Playing) {
            self.board_mut()?
                .can_upgrade_settlement_to_city(vertex, player_id)?;

            self.get_player_mut(player_id)?.pay(&Cost::CITY)?;

            self.board_mut()?
                .upgrade_settlement_to_city(vertex, player_id)?;
            Ok(())
        } else {
            Err(GameError::GameIsNotPlaying)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InvalidAction::{
        EdgeOccupied, NotConnected, NotYourSettlement, RoadMustStartFromNewSettlement,
        TooCloseToBuilding, UnexistingEdge, UnexistingVertex,
    };
    use crate::board::BuildingKind::{City, Settlement};
    use crate::player::PlayerColor;
    use crate::{Building, Resource, ResourceCounts};

    fn init_game() -> Game {
        let scenario = Scenario::test_scenario();
        let terrains = scenario.terrains().to_vec();
        let mut game = Game::new(
            scenario,
            vec![
                Player::new(PlayerColor::Blue),
                Player::new(PlayerColor::Red),
            ],
        )
        .unwrap();

        game.start(&terrains).unwrap();
        game.next_player();
        game.build_settlement(PlayerId::new(1), VertexId::new(4))
            .unwrap();
        game.build_settlement(PlayerId::new(1), VertexId::new(8))
            .unwrap();
        game.build_road(PlayerId::new(1), EdgeId::new(5)).unwrap();

        game.set_status(GameStatus::Playing);
        game.players[1].receive(Resource::Wheat, 2);
        game.players[1].receive(Resource::Stone, 3);
        game.upgrade_settlement_to_city(PlayerId::new(1), VertexId::new(4))
            .unwrap();
        game.set_status(GameStatus::Placement(Placement::new(1).unwrap()));
        game.next_player();

        game
    }

    #[test]
    fn test_apply_roll_with_production() {
        let mut game = init_game();
        println!("{:?}", game.board());
        game.apply_roll(Roll::new(2, 4).unwrap()).unwrap();
        assert_eq!(
            game.players[1].hand().resources(),
            ResourceCounts::new([2, 0, 0, 0, 0])
        );
    }

    #[test]
    fn test_apply_roll_with_robber() {
        let mut game = init_game();
        game.players[0].receive(Resource::Wood, 3);
        game.players[0].receive(Resource::Wheat, 2);
        game.players[0].receive(Resource::Stone, 2);
        assert_eq!(
            game.apply_roll(Roll::new(4, 3).unwrap()),
            Ok(RollOutcome::RobberActivated {
                must_discard: vec![]
            })
        );
        game.players[0].receive(Resource::Brick, 1);
        assert_eq!(
            game.apply_roll(Roll::new(4, 3).unwrap()),
            Ok(RollOutcome::RobberActivated {
                must_discard: vec![PlayerId::new(0)]
            })
        )
    }

    #[test]
    fn test_build_road() {
        let mut game = init_game();
        assert_eq!(
            game.build_road(PlayerId::new(0), EdgeId::new(0)),
            Err(GameError::Placement(RoadMustStartFromNewSettlement))
        );
        game.next_player();
        assert_eq!(
            game.build_road(PlayerId::new(1), EdgeId::new(0)),
            Err(GameError::Placement(RoadMustStartFromNewSettlement))
        );
        assert_eq!(
            game.build_road(PlayerId::new(1), EdgeId::new(7)),
            Err(GameError::Placement(RoadMustStartFromNewSettlement))
        );
        assert_eq!(
            game.build_road(PlayerId::new(1), EdgeId::new(4)),
            Err(GameError::Placement(RoadMustStartFromNewSettlement))
        );
        assert!(game.build_road(PlayerId::new(1), EdgeId::new(10)).is_ok()); //Attenttion nouvelle route de placée
        assert_eq!(game.board().unwrap().roads()[10], Some(PlayerId::new(1)));
        game.set_status(GameStatus::Playing);

        game.set_status(GameStatus::Playing);
        assert_eq!(
            game.build_road(PlayerId::new(1), EdgeId::new(10)),
            Err(GameError::Placement(EdgeOccupied(EdgeId::new(10))))
        );
        assert_eq!(
            game.build_road(PlayerId::new(1), EdgeId::new(6)),
            Err(GameError::NotEnoughResources)
        );
        assert_eq!(
            game.build_road(PlayerId::new(1), EdgeId::new(11)),
            Err(GameError::Placement(NotConnected))
        );
        assert_eq!(
            game.build_road(PlayerId::new(1), EdgeId::new(114)),
            Err(GameError::Placement(UnexistingEdge(EdgeId::new(114))))
        );
        game.players[1].receive(Resource::Brick, 1);
        game.players[1].receive(Resource::Wood, 1);
        assert_eq!(game.build_road(PlayerId::new(1), EdgeId::new(6)), Ok(()));
        assert_eq!(game.board().unwrap().roads()[6], Some(PlayerId::new(1)));
        game.players[1].receive(Resource::Brick, 1);
        game.players[1].receive(Resource::Wood, 1);
        assert!(game.build_road(PlayerId::new(1), EdgeId::new(7)).is_ok());
        assert_eq!(
            game.build_road(PlayerId::new(1), EdgeId::new(7)),
            Err(GameError::Placement(EdgeOccupied(EdgeId::new(7))))
        );
        game.set_status(GameStatus::End);
        assert_eq!(
            game.build_road(PlayerId::new(1), EdgeId::new(6)),
            Err(GameError::GameOver)
        );
    }

    #[test]
    fn test_build_settlement() {
        let mut game = init_game();
        assert_eq!(
            game.build_settlement(PlayerId::new(0), VertexId::new(9)),
            Err(GameError::Placement(TooCloseToBuilding(VertexId::new(9))))
        );
        assert_eq!(
            game.build_settlement(PlayerId::new(0), VertexId::new(100)),
            Err(GameError::Placement(UnexistingVertex(VertexId::new(100))))
        );
        assert!(
            game.build_settlement(PlayerId::new(0), VertexId::new(0))
                .is_ok()
        );
        assert_eq!(
            game.board().unwrap().buildings()[0],
            Some(Building::new(Settlement, PlayerId::new(0)))
        );
        game.build_road(PlayerId::new(0), EdgeId::new(8)).unwrap();

        game.set_status(GameStatus::Playing);
        game.players[0].receive(Resource::Brick, 1);
        game.players[0].receive(Resource::Wood, 1);
        game.build_road(PlayerId::new(0), EdgeId::new(11)).unwrap();
        assert_eq!(
            game.build_settlement(PlayerId::new(0), VertexId::new(9)),
            Err(GameError::Placement(TooCloseToBuilding(VertexId::new(9))))
        );
        assert_eq!(
            game.build_settlement(PlayerId::new(0), VertexId::new(100)),
            Err(GameError::Placement(UnexistingVertex(VertexId::new(100))))
        );
        assert_eq!(
            game.build_settlement(PlayerId::new(0), VertexId::new(11)),
            Err(GameError::Placement(NotConnected))
        );
        assert_eq!(
            game.build_settlement(PlayerId::new(0), VertexId::new(10)),
            Err(GameError::NotEnoughResources)
        );
        game.players[0].receive(Resource::Brick, 1);
        game.players[0].receive(Resource::Wood, 1);
        game.players[0].receive(Resource::Wheat, 1);
        game.players[0].receive(Resource::Wool, 1);
        assert_eq!(
            game.build_settlement(PlayerId::new(0), VertexId::new(10)),
            Ok(())
        );
        assert_eq!(
            game.board().unwrap().buildings()[10],
            Some(Building::new(Settlement, PlayerId::new(0)))
        );

        game.set_status(GameStatus::End);
        assert_eq!(
            game.build_settlement(PlayerId::new(0), VertexId::new(12)),
            Err(GameError::GameOver)
        );
    }

    #[test]
    fn test_upgrade_settlement_to_city() {
        let mut game = init_game();
        assert_eq!(
            game.upgrade_settlement_to_city(PlayerId::new(0), VertexId::new(8)),
            Err(GameError::GameIsNotPlaying)
        );
        game.set_status(GameStatus::Playing);
        assert_eq!(
            game.upgrade_settlement_to_city(PlayerId::new(18), VertexId::new(8)),
            Err(GameError::NotYourTurn)
        );

        assert_eq!(
            game.upgrade_settlement_to_city(PlayerId::new(0), VertexId::new(8)),
            Err(GameError::Placement(NotYourSettlement(VertexId::new(8))))
        );
        game.next_player();
        assert_eq!(
            game.upgrade_settlement_to_city(PlayerId::new(1), VertexId::new(100)),
            Err(GameError::Placement(UnexistingVertex(VertexId::new(100))))
        );
        assert_eq!(
            game.upgrade_settlement_to_city(PlayerId::new(1), VertexId::new(8)),
            Err(GameError::NotEnoughResources)
        );
        game.players[1].receive(Resource::Wheat, 2);
        game.players[1].receive(Resource::Stone, 3);
        assert_eq!(
            game.upgrade_settlement_to_city(PlayerId::new(1), VertexId::new(8)),
            Ok(())
        );
        assert_eq!(
            game.board().unwrap().buildings()[4],
            Some(Building::new(City, PlayerId::new(1)))
        );

        game.set_status(GameStatus::End);
        assert_eq!(
            game.upgrade_settlement_to_city(PlayerId::new(1), VertexId::new(4)),
            Err(GameError::GameOver)
        );
    }

    #[test]
    fn test_players_order() {
        let mut game = Game::new(
            Scenario::test_scenario(),
            vec![
                Player::new(PlayerColor::Blue),
                Player::new(PlayerColor::White),
                Player::new(PlayerColor::Red),
                Player::new(PlayerColor::Orange),
            ],
        )
        .unwrap();

        assert_eq!(
            game.set_players_order(vec![
                Roll::new(1, 1).unwrap(),
                Roll::new(6, 2).unwrap(),
                Roll::new(3, 3).unwrap()
            ]),
            Err(GameError::WrongRollCount)
        );
        assert_eq!(
            game.set_players_order(vec![
                Roll::new(1, 1).unwrap(),
                Roll::new(6, 2).unwrap(),
                Roll::new(3, 3).unwrap(),
                Roll::new(4, 4).unwrap()
            ]),
            Err(GameError::TiedRolls)
        );
        assert_eq!(
            game.set_players_order(vec![
                Roll::new(1, 1).unwrap(),
                Roll::new(3, 2).unwrap(),
                Roll::new(3, 3).unwrap(),
                Roll::new(4, 4).unwrap()
            ]),
            Ok(())
        );
        assert_eq!(
            game.turn_order,
            vec![
                PlayerId::new(3),
                PlayerId::new(0),
                PlayerId::new(1),
                PlayerId::new(2)
            ]
        );
        assert_eq!(
            game.set_players_order(vec![
                Roll::new(1, 1).unwrap(),
                Roll::new(6, 4).unwrap(),
                Roll::new(3, 3).unwrap(),
                Roll::new(4, 4).unwrap()
            ]),
            Ok(())
        );
        assert_eq!(
            game.turn_order,
            vec![
                PlayerId::new(1),
                PlayerId::new(2),
                PlayerId::new(3),
                PlayerId::new(0)
            ]
        );
    }

    #[test]
    fn test_turn_order() {
        let mut game = Game::new(
            Scenario::test_scenario(),
            vec![
                Player::new(PlayerColor::Blue),
                Player::new(PlayerColor::White),
                Player::new(PlayerColor::Red),
                Player::new(PlayerColor::Orange),
            ],
        )
        .unwrap();
        assert_eq!(
            game.set_players_order(vec![
                Roll::new(1, 1).unwrap(),
                Roll::new(3, 2).unwrap(),
                Roll::new(3, 3).unwrap(),
                Roll::new(4, 4).unwrap()
            ]),
            Ok(())
        );
        assert_eq!(
            game.turn_order,
            vec![
                PlayerId::new(3),
                PlayerId::new(0),
                PlayerId::new(1),
                PlayerId::new(2)
            ]
        );
        assert_eq!(game.current_player(), PlayerId::new(3));
        assert_eq!(game.check_player(PlayerId::new(3)), Ok(()));
        game.next_player();
        assert_eq!(game.current_player(), PlayerId::new(0));
        assert_eq!(game.check_player(PlayerId::new(0)), Ok(()));

        game.next_player();
        assert_eq!(game.current_player(), PlayerId::new(1));
        assert_eq!(game.check_player(PlayerId::new(1)), Ok(()));

        game.next_player();
        assert_eq!(game.current_player(), PlayerId::new(2));
        assert_eq!(game.check_player(PlayerId::new(2)), Ok(()));

        game.next_player();
        assert_eq!(game.current_player(), PlayerId::new(3));
        assert_eq!(game.check_player(PlayerId::new(3)), Ok(()));
    }
}
