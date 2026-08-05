use crate::Resource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceCounts([u8; 5]);

impl ResourceCounts {
    pub const fn new(counts: [u8; 5]) -> Self {
        Self(counts)
    }
    pub(crate) fn amount(&self, r: Resource) -> u8 {
        self.0[r.index()]
    }
    pub(crate) fn add(&mut self, resources : ResourceCounts) {
        resources.0.iter().enumerate().for_each(|(i, &n)| self.0[i] += n);
    }
    pub(crate) fn count(&self) -> u8 {
        self.0.iter().copied().sum()
    }

    pub(crate) fn covers(&self, other: &Self) -> bool {
        self.0
            .iter()
            .zip(other.0.iter())
            .all(|(mine, needed)| mine >= needed)
    }
    pub(crate) fn try_subtract(&mut self, other: &Self) -> bool {
        if self.covers(other) {
            for (mine, taken) in self.0.iter_mut().zip(other.0.iter()) {
                *mine -= taken;
            }
            true
        } else {
            false
        }
    }

    pub(crate) fn get_resource(&self, index : u8) -> ResourceCounts {
        let mut index = index % self.count() + 1;
        let r_index = self.0.iter().enumerate().map(|(i, &amount)| if index <= amount {true} else {index -= amount; false}).position(|v| v).unwrap();
        let mut result : [u8;5] = [0; 5];
        result[r_index] = 1;
        ResourceCounts::new(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::Cost;
    use super::*;

    #[test]
    fn test_resource_counts() {
        let mut counts = ResourceCounts::default();
        counts.add(ResourceCounts::new([1, 0, 0, 0, 0]));
        assert_eq!(counts.0, [1, 0, 0, 0, 0]);
        counts.add(ResourceCounts::new([4, 0, 0, 0, 0]));
        assert_eq!(counts.0, [5, 0, 0, 0, 0]);
        counts.add(ResourceCounts::new([0, 1, 0, 0, 0]));
        assert_eq!(counts.0, [5, 1, 0, 0, 0]);
    }

    #[test]
    fn test_resource_counts_covers() {
        let mut counts = ResourceCounts::default();
        counts.add(ResourceCounts::new([1, 0, 0, 0, 0]));
        assert!(!counts.covers(&Cost::ROAD.resources()));
        counts.add(ResourceCounts::new([0, 0, 4, 0, 0]));
        assert!(counts.covers(&Cost::ROAD.resources()));
    }

    #[test]
    fn test_resource_counts_try_subtract() {
        let mut counts = ResourceCounts::default();
        counts.add(ResourceCounts::new([1, 0, 0, 0, 0]));
        counts.add(ResourceCounts::new([0, 0, 1, 0, 0]));
        assert!(counts.try_subtract(&Cost::ROAD.resources()));
        assert_eq!(counts, ResourceCounts::default())
    }

    #[test]
    fn test_get_resource() {
        let resources = ResourceCounts([0,1,2,5,0]);
        assert_eq!(resources.get_resource(0), ResourceCounts::new([0,1,0,0,0]));
        assert_eq!(resources.get_resource(1), ResourceCounts::new([0,0,1,0,0]));
        assert_eq!(resources.get_resource(2), ResourceCounts::new([0,0,1,0,0]));
        assert_eq!(resources.get_resource(8), ResourceCounts::new([0,1,0,0,0]));
    }


}
