use crate::resource::counts::ResourceCounts;

pub struct Cost(ResourceCounts);

impl Cost {
    
    pub fn resources(&self) -> ResourceCounts {
        self.0
    }

    pub const ROAD : Cost = Cost(ResourceCounts::new([1, 0, 1, 0, 0]));
    pub const SETTLEMENT : Cost = Cost(ResourceCounts::new([1, 0, 1, 1, 1]));
    pub const CITY : Cost = Cost(ResourceCounts::new([0, 3, 0, 2, 0]));
    pub const DEV_CARD : Cost = Cost(ResourceCounts::new([0, 1, 0, 1, 1]));
}