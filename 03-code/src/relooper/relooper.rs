use crate::middle_end::ir::Program;
use crate::relooper::soupify::soupify;

pub fn reloop(prog: Box<Program>) {
    soupify(prog);
}
