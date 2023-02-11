use std::collections::HashSet;

use crate::backend::stack_allocation::var_locations::{VarLocation, VarLocations};
use crate::data_structures::interval_tree::{IntervalTree, Mergeable};
use crate::middle_end::ids::VarId;

struct ClashList {
    clashes: HashSet<VarId>,
}

impl Mergeable for ClashList {
    fn merge(&mut self, other: &Self) {
        self.clashes.extend(other.clashes.to_owned())
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

    fn insert(&mut self, location: VarLocation) {}
}
