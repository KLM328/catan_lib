use std::collections::HashMap;
use crate::board::production;
use crate::player::PlayerId;
use crate::resource::Resource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gain {
    pub player: PlayerId,
    pub resource: Resource,
    pub amount: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Production {
    gains: Vec<Gain>,
}

impl Production {
    pub(crate) fn new(entries: &[(PlayerId, [u8; 5])]) -> Self {
        let mut production = Self::default();
        for &(player, counts) in entries {
            for (index, &amount) in counts.iter().enumerate() {
                if amount > 0 {
                    production.add_gain(Gain {
                        player,
                        resource: Resource::from_index(index).unwrap(),
                        amount,
                    });
                }
            }
        }
        production.sort();
        production
    }

    pub(crate) fn add_gain(&mut self, gain: Gain) {
        if let Some(existing) = self.gains.iter_mut()
            .find(|g| g.player == gain.player && g.resource == gain.resource)
        {
            existing.amount += gain.amount;
        } else {
            self.gains.push(gain);
        }
    }

    pub(crate) fn gains(&self) -> &[Gain] { &self.gains }

    pub fn sort(&mut self) {
        self.gains.sort_by_key(|g| (g.player.value(), g.resource.index()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_add_gain() {
        let mut production = Production::default();
        production.add_gain(Gain { player: PlayerId::new(0), resource: Resource::Wood, amount: 2 });
        assert_eq!(production.gains.len(), 1);
    }

    #[test]
    fn test_production_is_empty() {
        let production = Production::default();
        assert!(production.gains.is_empty());    }

    #[test]
    fn test_production_gains() {
        let mut production = Production::default();
        production.add_gain(Gain { player: PlayerId::new(0), resource: Resource::Wood, amount: 2 });
        assert_eq!(production.gains()[0], Gain { player: PlayerId::new(0), resource: Resource::Wood, amount: 2 });
    }
}