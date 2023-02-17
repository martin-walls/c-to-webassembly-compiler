use std::collections::HashSet;

use crate::back_end::dataflow_analysis::clash_graph::ClashGraph;
use crate::back_end::stack_allocation::var_locations::{VarLocation, VarLocations};
use crate::middle_end::ids::VarId;

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

impl NaiveVarLocations {
    /// Return just the locations that overlap with the given location
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
}

impl VarLocations for NaiveVarLocations {
    fn into_hashset(self) -> HashSet<VarLocation> {
        self.locations
    }

    fn find_lowest_non_clashing_location_for_var(
        &self,
        var: VarId,
        byte_size: u64,
        clash_graph: &ClashGraph,
    ) -> VarLocation {
        let mut lowest_possible_location = VarLocation {
            var: var.to_owned(),
            start: 0,
            byte_size: byte_size as u32,
        };
        let mut is_valid_allocation = false;
        while !is_valid_allocation {
            is_valid_allocation = true;
            // check against all existing allocations for clashes
            for existing_location in self.get_locations_overlapping_with(&lowest_possible_location)
            {
                // check if the var clashes with the overlapping var allocation
                let do_vars_clash = clash_graph.do_vars_clash(&var, &existing_location.var);
                if do_vars_clash {
                    // move the var we're allocating to the next addr past the var it clashes with
                    lowest_possible_location.start = existing_location.end_exclusive();
                    is_valid_allocation = false;
                    // restart checking against all existing allocations, now that we've moved where
                    // we're trying to allocate to
                    break;
                }
            }
        }
        lowest_possible_location
    }

    fn insert(&mut self, location: VarLocation, _clash_graph: &ClashGraph) {
        self.locations.insert(location);
    }
}
