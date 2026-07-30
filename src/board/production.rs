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
    pub fn add_gain(&mut self, gain: Gain) { self.gains.push(gain) }
    pub fn gains(&self) -> &[Gain] { &self.gains }
    pub fn is_empty(&self) -> bool { self.gains.is_empty() }
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
        assert!(production.is_empty());
    }

    #[test]
    fn test_production_gains() {
        let mut production = Production::default();
        production.add_gain(Gain { player: PlayerId::new(0), resource: Resource::Wood, amount: 2 });
        assert_eq!(production.gains()[0], Gain { player: PlayerId::new(0), resource: Resource::Wood, amount: 2 });
    }
}