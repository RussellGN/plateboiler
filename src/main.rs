//! # Description
//! This CLI program helps setup various types of dev projects, think
//! npm projects and the likes.
//!
//! ## Features
//! For the time being. It will only setup web-app projects using npm and vite, as well as python django projects.
//!
//! ## Workflow
//! walks you through prompts asking for the type of project you want set up and any dependancies along with it,
//! similar to more specific framework CLIs

use std::process;

use plateboiler::{clear_terminal, red_log, yellow_log};
use plateboiler::{get_program_args, run_program};

const ERROR_EXIT_CODE: i32 = 0; // Not an error exit code, I know. Using it so that terminal doesnt print extra text on-exit

fn main() {
    clear_terminal();
    yellow_log("-----------------------------------------");

    let args = match get_program_args() {
        Ok(args) => args,
        Err(e) => {
            red_log(format!("Error: {} \nExiting...", e.msg()).as_str());
            process::exit(ERROR_EXIT_CODE)
        }
    };

    match run_program(args) {
        Ok(msg) => println!("{msg}"),
        Err(e) => {
            red_log(format!("Error: {} \nExiting...", e.msg()).as_str());
            process::exit(ERROR_EXIT_CODE)
        }
    }

    println!("");
}
