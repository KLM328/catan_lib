use crate::Resource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceCounts([u8; 5]);

impl ResourceCounts {
    pub const fn new(counts: [u8; 5]) -> Self {
        Self(counts)
    }
    pub fn amount(&self, r: Resource) -> u8 {
        self.0[r.index()]
    }
    pub fn add(&mut self, r: Resource, n: u8) {
        self.0[r.index()] += n;
    }
    pub fn count(&self) -> u8 {
        self.0.iter().map(|&n| n).sum()
    }

    pub fn covers(&self, other: &Self) -> bool {
        self.0
            .iter()
            .zip(other.0.iter())
            .all(|(mine, needed)| mine >= needed)
    }
    pub fn try_subtract(&mut self, other: &Self) -> bool {
        if self.covers(other) {
            for (mine, taken) in self.0.iter_mut().zip(other.0.iter()) {
                *mine -= taken;
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Cost;
    use super::*;

    #[test]
    fn test_resource_counts() {
        let mut counts = ResourceCounts::default();
        counts.add(Resource::Wood, 1);
        assert_eq!(counts.0, [1, 0, 0, 0, 0]);
        counts.add(Resource::Wood, 4);
        assert_eq!(counts.0, [5, 0, 0, 0, 0]);
        counts.add(Resource::Stone, 1);
        assert_eq!(counts.0, [5, 1, 0, 0, 0]);
    }

    #[test]
    fn test_resource_counts_covers() {
        let mut counts = ResourceCounts::default();
        counts.add(Resource::Wood, 1);
        assert!(!counts.covers(&Cost::ROAD.resources()));
        counts.add(Resource::Brick, 4);
        assert!(counts.covers(&Cost::ROAD.resources()));
    }

    #[test]
    fn test_resource_counts_try_subtract() {
        let mut counts = ResourceCounts::default();
        counts.add(Resource::Wood, 1);
        counts.add(Resource::Brick, 1);
        assert!(counts.try_subtract(&Cost::ROAD.resources()));
        assert_eq!(counts, ResourceCounts::default())
    }


}
