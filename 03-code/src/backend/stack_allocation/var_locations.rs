use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;

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

    fn overlaps(&self, other: &VarLocation) -> bool {
        // no overlap if one interval ends before another starts
        // end() is exclusive, so use <=
        let no_overlap = self.end() <= other.start || other.end() <= self.start;

        !no_overlap
    }

    /// Returns true if this interval is strictly after the other interval
    fn is_strictly_after(&self, other: &VarLocation) -> bool {
        // end() is exclusive, so use >=
        self.start >= other.end()
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

    /// Return just the locations that overlap with the given interval
    fn get_locations_overlapping_with(
        &self,
        overlap_location: &VarLocation,
    ) -> HashSet<&VarLocation>;

    /// Add a new location to the data structure
    fn insert(&mut self, location: VarLocation);
}

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
