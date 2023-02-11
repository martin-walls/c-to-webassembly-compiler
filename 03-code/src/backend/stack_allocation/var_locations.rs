use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;

use crate::backend::dataflow_analysis::clash_graph::ClashGraph;
use crate::middle_end::ids::VarId;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct VarLocation {
    pub var: VarId,
    pub start: u32,
    pub byte_size: u32,
}

impl VarLocation {
    /// End of the interval (exclusive)
    pub fn end(&self) -> u32 {
        self.start + self.byte_size
    }

    /// End of the interval (inclusive)
    pub fn end_inclusive(&self) -> u32 {
        self.start + self.byte_size - 1
    }

    pub fn overlaps(&self, other: &VarLocation) -> bool {
        // no overlap if one interval ends before another starts
        // end() is exclusive, so use <=
        let no_overlap = self.end() <= other.start || other.end() <= self.start;

        !no_overlap
    }
}

impl fmt::Display for VarLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: [{}, {})", self.var, self.start, self.end())
    }
}

pub trait VarLocations {
    /// Get the allocations of all vars
    fn into_hashset(self) -> HashSet<VarLocation>;

    /// Find the lowest valid location to allocate a new variable so that it doesn't clash
    /// with any variables already allocated.
    fn find_lowest_non_clashing_location_for_var(
        &self,
        var: VarId,
        byte_size: u64,
        clash_graph: &ClashGraph,
    ) -> VarLocation;

    /// Add a new location to the data structure
    fn insert(&mut self, location: VarLocation, clash_graph: &ClashGraph);
}
