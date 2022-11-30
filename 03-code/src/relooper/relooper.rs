use crate::middle_end::ir::Program;
use crate::relooper::soupify::soupify;

pub fn reloop(prog: &mut Box<Program>) {
    soupify(prog);
}
