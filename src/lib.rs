#![allow(dead_code)]
#![allow(unused_parens)]

#[macro_use]
extern crate lalrpop_util;

use std::error::Error;
use std::path::Path;

use clap::Parser as ClapParser;
use log::{debug, info, trace};

use front_end::parser::parse;
use middle_end::ast_to_ir::convert_to_ir;
use preprocessor::preprocess;
use program_config::enabled_optimisations::EnabledOptimisations;
use program_config::enabled_profiling::EnabledProfiling;

use crate::back_end::target_code_generation::generate_target_code;
use crate::middle_end::middle_end_optimiser::ir_optimiser::optimise_ir;
use crate::relooper::relooper::reloop;

mod back_end;
mod data_structures;
mod fmt_indented;
mod front_end;
mod id;
mod middle_end;
mod preprocessor;
mod program_config;
mod relooper;

#[derive(ClapParser, Debug)]
pub struct CliConfig {
    /// The path to the input file to compile
    filepath: String,
    /// The path of the output file to generate
    #[arg(short, long, default_value_t = ("module.wasm".to_owned()))]
    output: String,

    /// Enable tail-call optimisation (default)
    #[arg(long, group = "group_opt_tailcall")]
    opt_tailcall: bool,
    /// Disable tail-call optimisation
    #[arg(long, group = "group_opt_tailcall")]
    noopt_tailcall: bool,

    /// Enable unreachable procedure elimination (default)
    #[arg(long, group = "group_opt_unreachable_procedure")]
    opt_unreachable_procedure: bool,
    /// Disable unreachable procedure elimination
    #[arg(long, group = "group_opt_unreachable_procedure")]
    noopt_unreachable_procedure: bool,

    /// Enable stack allocation optimisation (default)
    #[arg(long, group = "group_opt_stack_allocation")]
    opt_stack_allocation: bool,
    /// Disable stack allocation optimisation
    #[arg(long, group = "group_opt_stack_allocation")]
    noopt_stack_allocation: bool,

    /// Enable stack usage profiling
    #[arg(long, group = "group_prof_stack")]
    prof_stack: bool,
    /// Disable stack usage profiling (default)
    #[arg(long, group = "group_prof_stack")]
    noprof_stack: bool,
}

pub fn run(config: CliConfig) -> Result<(), Box<dyn Error>> {
    let enabled_optimisations = EnabledOptimisations::construct(&config);
    let enabled_profiling = EnabledProfiling::construct(&config);
    debug!("{:?}", enabled_optimisations);
    debug!("{:?}", enabled_profiling);

    // Run C preprocessor
    let source = preprocess(Path::new(&config.filepath))?;
    // Generate AST
    let ast = parse(source)?;
    // Convert AST to three-address code IR
    let mut ir = convert_to_ir(ast)?;
    trace!("Non-optimised IR: {}", ir);
    // Run optimisations on the IR
    optimise_ir(&mut ir, &enabled_optimisations)?;
    info!("Optimised IR: {}", ir);
    // Run the Relooper algorithm
    let relooped_ir = reloop(ir);
    // Generate target wasm code
    let wasm_module =
        generate_target_code(relooped_ir, &enabled_optimisations, &enabled_profiling)?;
    // write binary to file
    wasm_module.write_to_file(Path::new(&config.output))?;
    Ok(())
}
