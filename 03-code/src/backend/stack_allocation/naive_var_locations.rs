use std::collections::HashSet;

use crate::backend::stack_allocation::var_locations::{VarLocation, VarLocations};

pub struct NaiveVarLocations {
    locations: HashSet<VarLocation>,
}

impl NaiveVarLocations {
    pub fn new() -> Self {
        NaiveVarLocations {
            locations: HashSet::new(),
        }
    }
}

impl VarLocations for NaiveVarLocations {
    fn into_hashset(self) -> HashSet<VarLocation> {
        self.locations
    }

    fn get_locations_overlapping_with(
        &self,
        overlap_location: &VarLocation,
    ) -> HashSet<&VarLocation> {
        let mut overlaps = HashSet::new();

        for location in &self.locations {
            if !location.overlaps(overlap_location) {
                continue;
            }
            overlaps.insert(location);
        }

        overlaps
    }

    fn insert(&mut self, location: VarLocation) {
        self.locations.insert(location);
    }
}
