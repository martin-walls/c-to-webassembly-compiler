use std::collections::HashSet;

use crate::backend::dataflow_analysis::clash_graph::ClashGraph;
use crate::backend::stack_allocation::var_locations::{VarLocation, VarLocations};
use crate::data_structures::interval_tree::{Interval, IntervalTree, Mergeable};
use crate::middle_end::ids::VarId;

struct ClashList {
    clashes: HashSet<VarId>,
    universal_clash: bool,
}

impl Mergeable for ClashList {
    fn merge(&mut self, other: &Self) {
        self.clashes.extend(other.clashes.to_owned());
        self.universal_clash |= other.universal_clash;
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
        todo!()
    }

    fn insert(&mut self, location: VarLocation, clash_graph: &ClashGraph) {
        // insert clashes to interval tree
        let interval = Interval {
            start: location.start,
            end: location.end_inclusive(),
        };

        let clashes = ClashList {
            clashes: clash_graph.get_all_clashes(&location.var),
            universal_clash: clash_graph.does_var_clash_universally(&location.var),
        };

        self.interval_tree.insert_or_merge(interval, clashes);

        self.locations.insert(location);
    }
}
