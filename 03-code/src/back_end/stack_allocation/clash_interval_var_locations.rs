use std::cmp::Ordering;
use std::collections::HashSet;

use crate::back_end::dataflow_analysis::clash_graph::ClashGraph;
use crate::back_end::stack_allocation::var_locations::{VarLocation, VarLocations};
use crate::middle_end::ids::VarId;

pub struct ClashIntervalVarLocations {
    locations: HashSet<VarLocation>,
    clash_intervals: Vec<ClashInterval>,
}

impl ClashIntervalVarLocations {
    pub fn new() -> Self {
        Self {
            locations: HashSet::new(),
            clash_intervals: Vec::new(),
        }
    }

    fn insert_clash_interval(&mut self, clash_interval: ClashInterval) {
        if self.clash_intervals.is_empty() {
            self.clash_intervals.push(clash_interval);
            return;
        }

        let mut low = 0_usize;
        let mut high = self.clash_intervals.len();

        while low < high {
            let mid = (high + low) / 2;

            let mid_interval = self.clash_intervals.get_mut(mid).expect("We've already checked that the vec is non-empty, therefore there is always a middle element");

            if &clash_interval == mid_interval {
                mid_interval.merge(clash_interval);
                return;
            } else if &clash_interval < mid_interval {
                // recurse to the left
                high = mid;
            } else {
                // clash_interval > mid_interval
                // recurse to the right
                low = mid + 1;
            }
        }

        // at this point, low == high
        assert_eq!(low, high);
        self.clash_intervals.insert(low, clash_interval);
    }
}

impl VarLocations for ClashIntervalVarLocations {
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
        'outer: loop {
            let mut i = 0_usize;
            loop {
                match self.clash_intervals.get(i) {
                    None => {
                        // no more clash intervals to check, so we are ok to allocate since we've
                        // got here without clashing
                        break 'outer;
                    }
                    Some(clash_interval) => {
                        if clash_interval.is_after(&lowest_possible_location) {
                            // all further clash intervals are past the end of the location we're checking
                            break 'outer;
                        }

                        if clash_interval.overlaps(&lowest_possible_location) {
                            // check if var clashes with existing clashes
                            let does_clash = clash_interval
                                .does_var_clash(&lowest_possible_location.var, clash_graph);

                            // if so, move lowest_possible_location past the end of the interval it clashes with, and restart the while loop
                            if does_clash {
                                // move lowest_possible_location
                                lowest_possible_location.start = clash_interval.end + 1;
                                continue 'outer;
                            }
                        }
                        i += 1;
                    }
                }
            }
        }
        // once we've left the loop, the location we're at is the lowest possible
        // location that doesn't clash
        lowest_possible_location
    }

    fn insert(&mut self, location: VarLocation, clash_graph: &ClashGraph) {
        let clash_interval = ClashInterval {
            start: location.start,
            end: location.end_inclusive(),
            clashes: clash_graph.get_all_clashes(&location.var),
            universal_clash: clash_graph.does_var_clash_universally(&location.var),
        };
        self.insert_clash_interval(clash_interval);
        self.locations.insert(location);
    }
}

struct ClashInterval {
    // inclusive
    start: u32,
    // inclusive
    end: u32,
    clashes: HashSet<VarId>,
    universal_clash: bool,
}

impl ClashInterval {
    fn overlaps(&self, var_location: &VarLocation) -> bool {
        self.start <= var_location.end_inclusive() && var_location.start <= self.end
    }

    fn is_after(&self, var_location: &VarLocation) -> bool {
        self.start > var_location.end_inclusive()
    }

    fn does_var_clash(&self, var: &VarId, clash_graph: &ClashGraph) -> bool {
        self.clashes.contains(var)
            || self.universal_clash
            || clash_graph.does_var_clash_universally(var)
    }

    fn merge(&mut self, other: Self) {
        self.clashes.extend(other.clashes);
        self.universal_clash |= other.universal_clash;
    }
}

impl PartialEq for ClashInterval {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl PartialOrd for ClashInterval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.start < other.start {
            Some(Ordering::Less)
        } else if self.start > other.start {
            Some(Ordering::Greater)
        } else if self.end < other.end {
            Some(Ordering::Less)
        } else if self.end > other.end {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }
}
