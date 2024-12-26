//! main binary functions
//! all data types and their implementations are in the `data` module

mod constants;
mod data;
mod utils;

pub use utils::{clear_terminal, red_log, yellow_log};

use std::env;

use data::{DidSomething, Flag, ProgramArguments, ProgramError};
use utils::PEResult;

pub fn get_program_args() -> PEResult<ProgramArguments> {
    let mut raw_args = env::args();
    raw_args.next(); // pop off executable path
    ProgramArguments::build(raw_args)
}

pub fn run_program(args: ProgramArguments) -> PEResult<&'static str> {
    if let DidSomething::Yes = Flag::handle_help_flag(&args) {
        return Ok("END OF HELP SECTION");
    };

    let project_type = args.get_project_type();
    if let Some(project_type) = project_type {
        project_type.check_for_required_tooling(&args.get_flags())?;
        project_type.set_up(args.get_flags())?;
        Ok("DONE")
    } else {
        Err(ProgramError::new(
            "No valid project type provided".to_string(),
        ))
    }
}
