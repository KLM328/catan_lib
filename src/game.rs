use crate::board::BuildingKind;
use crate::{
    Board, Cost, EdgeId, InvalidAction, InvalidBoard, NotEnoughResources, Player, PlayerId,
    Production, Resource, Roll, Scenario, Terrain, VertexId,
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
    TurnDrivenByPlacement,
    InvalidGameStatus,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameStatus {
    RobberIsMoving,
    Starting,
    FirstPlacementSettlement,
    FirstPlacementRoad,
    SecondPlacementSettlement,
    SecondPlacementRoad,
    Playing,
    End,
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
        self.playable_status(vec![GameStatus::Starting])?;
        self.board = Some(self.scenario.layout(shuffled)?);
        self.set_status(GameStatus::FirstPlacementSettlement);
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

    fn end_placement_turn(&mut self) {
        match self.status {
            GameStatus::FirstPlacementRoad => {
                if self.current_turn == self.turn_order.len() - 1 {
                    self.status = GameStatus::SecondPlacementSettlement;
                } else {
                    self.current_turn += 1;
                    self.status = GameStatus::FirstPlacementSettlement;
                }
            }
            GameStatus::SecondPlacementRoad => {
                if self.current_turn == 0 {
                    self.status = GameStatus::Playing;
                } else {
                    self.current_turn -= 1;
                    self.status = GameStatus::SecondPlacementSettlement;
                }
            }
            _ => {}
        }
    }

    pub fn next_player(&mut self) -> Result<(), GameError> {
        match self.status {
            GameStatus::Playing => {
                self.current_turn = (self.current_turn + 1) % self.turn_order.len();
                Ok(())
            }
            GameStatus::FirstPlacementSettlement
            | GameStatus::FirstPlacementRoad
            | GameStatus::SecondPlacementSettlement
            | GameStatus::SecondPlacementRoad => Err(GameError::TurnDrivenByPlacement),
            _ => Err(GameError::InvalidGameStatus),
        }
    }

    pub fn set_players_order(&mut self, rolls: Vec<Roll>) -> Result<(), GameError> {
        self.playable_status(vec![GameStatus::Starting])?;
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

    fn playable_status(&self, authorized_status: Vec<GameStatus>) -> Result<(), GameError> {
        if authorized_status.contains(&self.status) {
            Ok(())
        } else {
            Err(GameError::InvalidGameStatus)
        }
    }

    pub fn apply_roll(&mut self, roll: Roll) -> Result<RollOutcome, GameError> {
        self.playable_status(vec![GameStatus::Playing])?;
        let outcome = match roll.value() {
            7 => {
                self.set_status(GameStatus::RobberIsMoving);
                RollOutcome::RobberActivated {
                    must_discard: self
                        .players
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| p.hand().count() > 7)
                        .map(|(i, _)| PlayerId::new(i))
                        .collect::<Vec<PlayerId>>(),
                }
            }
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
        self.playable_status(vec![
            GameStatus::FirstPlacementRoad,
            GameStatus::SecondPlacementRoad,
            GameStatus::Playing,
        ])?;

        self.check_player(player_id)?;
        let current_status = self.status;

        self.board()?.can_place_road(self.status, edge, player_id)?;

        if matches!(self.status, GameStatus::Playing) {
            self.get_player_mut(player_id)?.pay(&Cost::ROAD)?;
        }

        self.board_mut()?
            .place_road(current_status, edge, player_id)?;
        self.end_placement_turn();
        Ok(())
    }

    pub fn build_settlement(
        &mut self,
        player_id: PlayerId,
        vertex: VertexId,
    ) -> Result<(), GameError> {
        self.playable_status(vec![
            GameStatus::FirstPlacementSettlement,
            GameStatus::SecondPlacementSettlement,
            GameStatus::Playing,
        ])?;

        self.check_player(player_id)?;

        let current_status = self.status;
        self.board()?
            .can_place_building(self.status, vertex, player_id)?;

        if matches!(self.status, GameStatus::Playing) {
            self.get_player_mut(player_id)?.pay(&Cost::SETTLEMENT)?;
        }
        self.board_mut()?
            .place_settlement(current_status, vertex, player_id)?;

        if matches!(self.status, GameStatus::SecondPlacementSettlement) {
            for r in self.board()?.resources_around(vertex) {
                self.get_player_mut(self.current_player())?.receive(r, 1);
            }
        }

        self.status = match self.status {
            GameStatus::FirstPlacementSettlement => GameStatus::FirstPlacementRoad,
            GameStatus::SecondPlacementSettlement => GameStatus::SecondPlacementRoad,
            other => other,
        };

        Ok(())
    }

    fn check_player(&self, player_id: PlayerId) -> Result<(), GameError> {
        if player_id == self.current_player() {
            Ok(())
        } else {
            Err(GameError::NotYourTurn)
        }
    }
    fn get_player_mut(&mut self, player_id: PlayerId) -> Result<&mut Player, GameError> {
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
        self.playable_status(vec![GameStatus::Playing])?;
        self.check_player(player_id)?;

        self.board_mut()?
            .can_upgrade_settlement_to_city(vertex, player_id)?;

        self.get_player_mut(player_id)?.pay(&Cost::CITY)?;

        self.board_mut()?
            .upgrade_settlement_to_city(vertex, player_id)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::PlayerColor;
    use crate::{NumberToken, ResourceCounts, Tile};

    #[test]
    fn partie_complete() {
        // 1. création + ordre des joueurs par les dés
        assert!(Game::new(Scenario::test_scenario(), vec![]).is_err());
        let mut game = Game::new(
            Scenario::test_scenario(),
            vec![
                Player::new(PlayerColor::White),
                Player::new(PlayerColor::Red),
                Player::new(PlayerColor::Blue),
            ],
        )
        .unwrap();

        assert_eq!(
            game.set_players_order(vec![Roll::new(2, 4).unwrap()]),
            Err(GameError::WrongRollCount)
        );
        assert_eq!(
            game.set_players_order(vec![
                Roll::new(2, 4).unwrap(),
                Roll::new(5, 2).unwrap(),
                Roll::new(3, 4).unwrap()
            ]),
            Err(GameError::TiedRolls)
        );
        assert_eq!(
            game.set_players_order(vec![
                Roll::new(2, 4).unwrap(),
                Roll::new(6, 2).unwrap(),
                Roll::new(3, 4).unwrap()
            ]),
            Ok(())
        );
        assert_eq!(
            game.turn_order,
            vec![PlayerId::new(1), PlayerId::new(2), PlayerId::new(0)]
        );
        assert_eq!(game.current_player(), PlayerId::new(1));

        assert_eq!(game.status(), GameStatus::Starting);

        // 2. start() avec un agencement fixe (terrains non mélangés)
        assert_eq!(
            game.start(&vec![]),
            Err(GameError::BoardInitialization(
                InvalidBoard::WrongTileCount {
                    expected: 3,
                    got: 0
                }
            ))
        );
        assert_eq!(
            game.start(&vec![Terrain::Desert]),
            Err(GameError::BoardInitialization(
                InvalidBoard::WrongTileCount {
                    expected: 3,
                    got: 1
                }
            ))
        );
        assert_eq!(
            game.start(&vec![Terrain::Desert, Terrain::Forest]),
            Err(GameError::BoardInitialization(
                InvalidBoard::WrongTileCount {
                    expected: 3,
                    got: 2
                }
            ))
        );
        assert_eq!(
            game.start(&vec![
                Terrain::Desert,
                Terrain::Forest,
                Terrain::Mountain,
                Terrain::Fields
            ]),
            Err(GameError::BoardInitialization(
                InvalidBoard::WrongTileCount {
                    expected: 3,
                    got: 4
                }
            ))
        );
        assert_eq!(
            game.start(&vec![Terrain::Desert, Terrain::Forest, Terrain::Mountain]),
            Err(GameError::BoardInitialization(
                InvalidBoard::WrongDistribution
            ))
        );
        assert_eq!(game.start(&game.scenario.terrains().to_vec()), Ok(()));
        assert_eq!(
            game.board().unwrap().tiles(),
            vec![
                Tile::Forest(NumberToken::new(6).unwrap()),
                Tile::Hills(NumberToken::new(8).unwrap()),
                Tile::Desert
            ]
        );

        game = Game::new(
            Scenario::standard(),
            vec![
                Player::new(PlayerColor::White),
                Player::new(PlayerColor::Red),
                Player::new(PlayerColor::Blue),
            ],
        )
        .unwrap();
        assert_eq!(
            game.set_players_order(vec![
                Roll::new(1, 4).unwrap(),
                Roll::new(6, 2).unwrap(),
                Roll::new(3, 4).unwrap()
            ]),
            Ok(())
        );
        assert_eq!(game.start(&game.scenario.terrains().to_vec()), Ok(()));

        // 3. phase de placement : boucle sur les 6 tours du serpentin
        //    - à chaque tour : vérifier current_player() et status()
        //    - poser colonie puis route
        //    - vérifier qu'une route avant la colonie est refusée
        assert_eq!(game.status(), GameStatus::FirstPlacementSettlement);
        assert_eq!(game.current_player(), PlayerId::new(1));
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(46)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.build_settlement(PlayerId::new(2), VertexId::new(12)),
            Err(GameError::NotYourTurn)
        );
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(120)),
            Err(GameError::Placement(InvalidAction::UnexistingVertex(
                VertexId::new(120)
            )))
        );

        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(1)),
            Ok(())
        );
        assert_eq!(game.current_player(), PlayerId::new(1));
        assert_eq!(game.status(), GameStatus::FirstPlacementRoad);
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(12)),
            Err(GameError::InvalidGameStatus)
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(899)),
            Err(GameError::Placement(InvalidAction::UnexistingEdge(
                EdgeId::new(899)
            )))
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(60)),
            Err(GameError::Placement(
                InvalidAction::RoadMustStartFromNewSettlement
            ))
        );
        assert_eq!(
            game.build_road(PlayerId::new(2), EdgeId::new(1)),
            Err(GameError::NotYourTurn)
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(1)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::FirstPlacementSettlement);
        assert_eq!(game.current_player(), PlayerId::new(2));
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(46)),
            Err(GameError::Placement(InvalidAction::TooCloseToBuilding(
                VertexId::new(1)
            )))
        );
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(1)),
            Err(GameError::Placement(InvalidAction::VertexOccupied(
                VertexId::new(1)
            )))
        );
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(11)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::FirstPlacementRoad);
        assert_eq!(game.current_player(), PlayerId::new(2));
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(1)),
            Err(GameError::Placement(InvalidAction::EdgeOccupied(
                EdgeId::new(1)
            )))
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(2)),
            Err(GameError::Placement(
                InvalidAction::RoadMustStartFromNewSettlement
            ))
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(19)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::FirstPlacementSettlement);
        assert_eq!(game.current_player(), PlayerId::new(0));
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(40)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::FirstPlacementRoad);
        assert_eq!(game.current_player(), PlayerId::new(0));
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(50)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::SecondPlacementSettlement);
        assert_eq!(game.current_player(), PlayerId::new(0));
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(20)),
            Ok(())
        );
        assert_eq!(
            game.players[game.current_player().value()]
                .hand()
                .resources(),
            ResourceCounts::new([1, 0, 0, 1, 1])
        );

        assert_eq!(game.status(), GameStatus::SecondPlacementRoad);
        assert_eq!(game.current_player(), PlayerId::new(0));
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(49)),
            Err(GameError::Placement(
                InvalidAction::RoadMustStartFromNewSettlement
            ))
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(46)),
            Err(GameError::Placement(
                InvalidAction::RoadMustStartFromNewSettlement
            ))
        );
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(23)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::SecondPlacementSettlement);
        assert_eq!(game.current_player(), PlayerId::new(2));
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(44)),
            Ok(())
        );
        assert_eq!(
            game.players[game.current_player().value()]
                .hand()
                .resources(),
            ResourceCounts::new([0, 0, 1, 1, 0])
        );

        assert_eq!(game.status(), GameStatus::SecondPlacementRoad);
        assert_eq!(game.current_player(), PlayerId::new(2));
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(54)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::SecondPlacementSettlement);
        assert_eq!(game.current_player(), PlayerId::new(1));
        assert_eq!(
            game.build_settlement(game.current_player(), VertexId::new(33)),
            Ok(())
        );
        assert_eq!(
            game.players[game.current_player().value()]
                .hand()
                .resources(),
            ResourceCounts::new([1, 1, 0, 0, 1])
        );

        assert_eq!(game.status(), GameStatus::SecondPlacementRoad);
        assert_eq!(game.current_player(), PlayerId::new(1));
        assert_eq!(
            game.build_road(game.current_player(), EdgeId::new(67)),
            Ok(())
        );

        assert_eq!(game.status(), GameStatus::Playing);
        assert_eq!(game.current_player(), PlayerId::new(1));

        // 6. quelques tours : apply_roll, build_road, next_player

        // 7. vérifier NotYourTurn pour un joueur hors tour
    }
}
