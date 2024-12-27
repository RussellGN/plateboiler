use colored::*;

use std::{
    env::consts,
    io::{self, Write},
    process::Command,
};

use crate::data::ProgramError;

pub type PEResult<T = ()> = Result<T, ProgramError>;

pub fn check_if_any_command_passes(cmds: &[&str]) -> Result<(), ()> {
    let mut check_result = Err(());
    for cmd in cmds {
        match run_seperate_cmd(cmd) {
            Ok(_) => check_result = Ok(()),
            Err(_) => (),
        }
    }
    check_result
}

pub fn run_seperate_cmd(cmd: &str) -> PEResult {
    if consts::OS == "linux" {
        let output = Command::new("sh").arg("-c").arg(cmd).output();
        if let Err(e) = output {
            return Err(ProgramError::new(format!("Error running `{cmd}`: {e}")));
        }
        Ok(())
    } else if consts::OS == "windows" {
        let output = Command::new("cmd").arg("/C").arg(cmd).output();
        if let Err(e) = output {
            return Err(ProgramError::new(format!("Error running `{cmd}`: {e}")));
        }
        Ok(())
    } else {
        Err(ProgramError::new(format!("OS not supported by CLI")))
    }
}

pub fn run_child_cmd(cmd: &str) -> PEResult {
    if consts::OS == "linux" {
        let status = Command::new("sh").arg("-c").arg(cmd).status();
        if let Err(e) = status {
            return Err(ProgramError::new(format!("Error running `{cmd}`: {e}")));
        }
        Ok(())
    } else if consts::OS == "windows" {
        let output = Command::new("cmd").arg("/C").arg(cmd).status();
        if let Err(e) = output {
            return Err(ProgramError::new(format!("Error running `{cmd}`: {e}")));
        }
        Ok(())
    } else {
        Err(ProgramError::new(format!("OS not supported by CLI")))
    }
}

pub fn clear_terminal() {
    let _ = run_child_cmd("cls");
    let _ = run_child_cmd("clear");
}

pub fn prompt_input(prompt: &str) -> PEResult<String> {
    print!("{}", prompt.underline());
    io::stdout()
        .flush()
        .expect("should be able to print buffered text to the console");
    let mut input = String::new();
    if let Err(e) = io::stdin().read_line(&mut input) {
        return Err(ProgramError::new(format!(
            "failed to read user input: {}",
            e.kind()
        )));
    };
    Ok(input)
}

// colored log functions
pub fn red_log(s: &str) {
    println!("{}", s.red());
}
pub fn blue_log(s: &str) {
    println!("{}", s.blue());
}
pub fn yellow_log(s: &str) {
    println!("{}", s.yellow());
}
pub fn green_log(s: &str) {
    println!("{}", s.green());
}
