use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;

use log::debug;

use crate::back_end::dataflow_analysis::clash_graph::ClashGraph;
use crate::back_end::stack_allocation::var_locations::{VarLocation, VarLocations};
use crate::data_structures::interval_tree::{Interval, IntervalTree, Mergeable};
use crate::middle_end::ids::VarId;

struct ClashList {
    clashes: HashSet<VarId>,
    universal_clash: bool,
}

impl ClashList {
    fn clashes_with(&self, var: &VarId) -> bool {
        self.universal_clash || self.clashes.contains(var)
    }
}

impl Mergeable for ClashList {
    fn merge(&mut self, other: Self) {
        self.clashes.extend(other.clashes);
        debug!("data: {:?}", self.clashes);
        self.universal_clash |= other.universal_clash;
    }
}

impl fmt::Display for ClashList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "clashes: ")?;
        for var in &self.clashes {
            write!(f, "{var}, ")?;
        }
        write!(f, "; universal clash: {}", self.universal_clash)
    }
}

pub struct IntervalTreeVarLocations {
    /// Store clashes in an interval tree.
    /// Each interval stores the union of all clashes of var locations that are allocated
    /// to that interval.
    interval_tree: IntervalTree<ClashList>,
    /// Store the actual var locations.
    locations: HashSet<VarLocation>,
}

impl IntervalTreeVarLocations {
    pub fn new() -> Self {
        Self {
            interval_tree: IntervalTree::new(),
            locations: HashSet::new(),
        }
    }
}

impl From<VarLocation> for Interval {
    fn from(location: VarLocation) -> Self {
        Self {
            start: location.start,
            end: location.end_inclusive(),
        }
    }
}

impl VarLocations for IntervalTreeVarLocations {
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

            for (interval, clashes) in self
                .interval_tree
                .find_overlaps(&lowest_possible_location.to_owned().into())
            {
                debug!("{lowest_possible_location} overlaps with {interval} ({clashes})");
                if clashes.clashes_with(&var) || clash_graph.does_var_clash_universally(&var) {
                    debug!("clashes");
                    is_valid_allocation = false;
                    // move the var we're allocating to the next addr past the var it clashes with
                    // interval.end is inclusive
                    lowest_possible_location.start = interval.end + 1;
                    // restart checking against all existing allocations, now that we've moved where
                    // we're trying to allocate to
                    break;
                }
            }
        }

        lowest_possible_location
    }

    fn insert(&mut self, location: VarLocation, clash_graph: &ClashGraph) {
        // insert clashes to interval tree
        let interval: Interval = location.to_owned().into();

        let clashes = ClashList {
            clashes: clash_graph.get_all_clashes(&location.var),
            universal_clash: clash_graph.does_var_clash_universally(&location.var),
        };

        self.interval_tree.insert_or_merge(interval, clashes);

        self.locations.insert(location);
    }
}
