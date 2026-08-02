use crate::board::BuildingKind;
use crate::{Board, Cost, EdgeId, InvalidAction, InvalidBoard, NotEnoughResources, Player, PlayerId, Production, Resource, Roll, Scenario, Terrain, VertexId};

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
    TurnDrivenByPlacement,
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
            GameStatus::Starting => Err(GameError::GameIsStarting),
            GameStatus::End => Err(GameError::GameOver),
            _ => Err(GameError::TurnDrivenByPlacement),
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
            GameStatus::End => Err(GameError::GameOver),
            _ => Ok(()),
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
        self.playable_status()?;
        self.check_player(player_id)?;

        let current_status = self.status;
        self.board()?.can_place_building(self.status, vertex, player_id)?;

        if matches!(self.status, GameStatus::Playing) {
            self.get_player_mut(player_id)?.pay(&Cost::SETTLEMENT)?;
        }
        self.board_mut()?.place_settlement(current_status, vertex, player_id)?;

        if matches!(self.status, GameStatus::SecondPlacementSettlement) {
            for r in self.board()?.resources_around(vertex) {
                self.get_player_mut(self.current_player())?.receive(r, 1);
            }
        }

        self.status = match self.status {
            GameStatus::FirstPlacementSettlement  => GameStatus::FirstPlacementRoad,
            GameStatus::SecondPlacementSettlement => GameStatus::SecondPlacementRoad,
            other => other,
        };

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

    #[test]
    fn partie_complete() {
        // 1. création + ordre des joueurs par les dés
        // 2. start() avec un agencement fixe (terrains non mélangés)
        // 3. phase de placement : boucle sur les 6 tours du serpentin
        //    - à chaque tour : vérifier current_player() et status()
        //    - poser colonie puis route
        //    - vérifier qu'une route avant la colonie est refusée
        // 4. vérifier le crédit des ressources de la 2e colonie
        // 5. status == Playing, current_player == premier joueur
        // 6. quelques tours : apply_roll, build_road, next_player
        // 7. vérifier NotYourTurn pour un joueur hors tour
    }
    
}
