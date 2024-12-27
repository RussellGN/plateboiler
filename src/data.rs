use colored::*;

use std::{
    env,
    fs::{self},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{
    constants::{
        CLI_HELP_TEXT_WITHOUT_PROJECT_NOR_FLAG_OPTION_DESCRIPTIONS, VALID_FLAGS,
        VALID_PROJECT_OPTIONS,
    },
    utils::{self, blue_log, green_log, prompt_input, yellow_log, PEResult},
};

#[derive(PartialEq)]
pub enum DidSomething {
    Yes,
    No,
}

#[derive(Debug)]
pub struct ProgramError {
    message: String,
}

// IMPORTANT! update enum values in tandem with constants::VALID_PROJECT_OPTIONS
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectType {
    Django,
    React,
    Next,
}

// IMPORTANT! update enum values in tandem with constants::VALID_FLAGS
#[derive(Debug, PartialEq, Clone)]
pub enum Flag {
    Help,
    Verbose,
    Name(Value),
    Test,
}

struct Terminal {
    working_dir: PathBuf,
    base_shell_args: [String; 2],
}

pub struct ProgramArguments {
    project_type: Option<ProjectType>,
    flags: Vec<Flag>,
}

#[derive(Debug, PartialEq)]
pub struct Value(pub Option<String>);

impl Clone for Value {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl ProgramArguments {
    pub fn build<T: Iterator<Item = String>>(mut raw_args: T) -> PEResult<Self> {
        let mut project_type: Option<ProjectType> = None;
        let mut flags: Vec<Flag> = vec![];

        while let Some(mut arg) = raw_args.next() {
            arg = arg.trim().to_lowercase();
            if arg.starts_with("-") {
                flags.push(Self::map_string_to_flag(arg)?);
            } else if project_type.is_none() {
                project_type = Some(Self::map_string_to_project_type(&arg)?);
            } else {
                return Err(ProgramError::new(format!(
                    "You can only provide one project type! Found extra type '{arg}'",
                )));
            }
        }

        Ok(Self {
            project_type,
            flags,
        })
    }

    pub fn get_project_type(&self) -> &Option<ProjectType> {
        &self.project_type
    }

    pub fn get_flags(&self) -> &Vec<Flag> {
        &self.flags
    }

    fn map_string_to_project_type(s: &str) -> PEResult<ProjectType> {
        let project_type = VALID_PROJECT_OPTIONS
            .iter()
            .find(|project_type| project_type.0 == s);

        if let Some(project_type) = project_type {
            Ok(project_type.1)
        } else {
            Err(ProgramError::new(format!(
                "'{s}' is not a valid project type, run again with --help or -h for more info."
            )))
        }
    }

    fn map_string_to_flag(s: String) -> PEResult<Flag> {
        let flag = VALID_FLAGS.iter().find(|flag| flag.0 == s || flag.1 == s);

        if let Some(flag) = flag {
            Ok(flag.2.to_owned())
        } else {
            Self::map_flag_with_value(s)
        }
    }

    fn map_flag_with_value(s: String) -> PEResult<Flag> {
        let s_split: Vec<_> = s.split("=").collect();
        let key = s_split[0];
        let value = s_split[1];

        let flag = VALID_FLAGS
            .iter()
            .find(|flag| key == flag.0 || key == flag.1);

        if let Some(flag) = flag {
            match flag.2 {
                Flag::Name(_) => Ok(Flag::Name(Value(Some(value.to_string())))),
                _ => Err(ProgramError::new(format!(
                    "'{key}' is not a valid flag, run again with --help or -h for more info."
                ))),
            }
        } else {
            Err(ProgramError::new(format!(
                "'{key}' is not a valid flag, run again with --help or -h for more info."
            )))
        }
    }
}

impl ProgramError {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn msg(&self) -> &str {
        &self.message
    }
}

impl ProjectType {
    pub fn set_up(&self, flags: &[Flag]) -> PEResult {
        Flag::log_if_verbose(format!("setting up {self:?} project").as_str(), flags);

        match self {
            ProjectType::Django => self.set_up_django_project(flags),
            ProjectType::React => self.set_up_react_project(flags),
            ProjectType::Next => self.set_up_next_project(),
        }
    }

    pub fn check_for_required_tooling(&self, flags: &[Flag]) -> PEResult {
        Flag::log_if_verbose(
            format!("checking required tooling for a {self:?} project...").as_str(),
            flags,
        );

        match self {
            ProjectType::Django => self.check_for_django_tooling(),
            ProjectType::React => self.check_for_react_tooling(),
            ProjectType::Next => self.check_for_next_tooling(),
        }
    }

    fn check_for_django_tooling(&self) -> PEResult {
        // check for python
        let cmds = ["python --version", "python3 -version"];
        if utils::check_if_any_command_passes(&cmds).is_err() {
            return Err(ProgramError::new(format!(
                "Could not confirm if python is installed, in order to set up a {self:?} project."
            )));
        }

        // check for python venv
        let cmds = ["python -m venv --help", "python3 -m venv --help"];
        if utils::check_if_any_command_passes(&cmds).is_err() {
            return Err(ProgramError::new(format!(
                "Could not confirm if the venv module is installed, in order to set up a {self:?} project."
            )));
        }

        // check for python pip
        let cmds = ["python -m pip --version", "python3 -m pip --version"];
        if utils::check_if_any_command_passes(&cmds).is_err() {
            return Err(ProgramError::new(format!(
                "Could not confirm if the pip package manager is installed, in order to set up a {self:?} project."
            )));
        };

        Ok(())
    }

    fn check_for_react_tooling(&self) -> PEResult {
        // check for node js
        let cmds = ["node --version"];
        if utils::check_if_any_command_passes(&cmds).is_err() {
            return Err(ProgramError::new(format!(
                "Could not confirm if Node js is installed, in order to set up a {self:?} project."
            )));
        }

        // check for npm
        let cmds = ["npm --version"];
        if utils::check_if_any_command_passes(&cmds).is_err() {
            return Err(ProgramError::new(format!(
                "Could not confirm if Npm is installed, in order to set up a {self:?} project."
            )));
        }

        Ok(())
    }

    fn check_for_next_tooling(&self) -> PEResult {
        // check for any of node, deno, bun
        let cmds = ["node --version || deno --version"];
        if utils::check_if_any_command_passes(&cmds).is_err() {
            return Err(ProgramError::new(format!(
                "Could not confirm if any of Node, or Deno is installed, in order to set up a {self:?} project."
            )));
        }

        // check for any of npm, yarn, pnpm, deno, bun
        let cmds = ["npm --version || yarn --version || pnpm --version || deno --version"];
        if utils::check_if_any_command_passes(&cmds).is_err() {
            return Err(ProgramError::new(format!(
                "Could not confirm if any of Npm, Yarn, Pnpm, or Deno is installed, in order to set up a {self:?} project."
            )));
        }

        Ok(())
    }

    fn set_up_django_project(&self, flags: &[Flag]) -> PEResult {
        // create dir
        let flag_set_proj_name = Flag::get_project_name(&flags);
        let mut proj_name = if let Some(s) = flag_set_proj_name {
            s
        } else {
            prompt_input("Enter project name: ")?
        };

        proj_name = proj_name.trim().to_string();
        Flag::log_if_verbose(format!("creating {proj_name:?} directory").as_str(), flags);

        let is_test_run = Flag::is_test_run(&flags);
        if is_test_run {
            proj_name = format!("test_runs/{proj_name}");
            let test_run_path = Path::new("test_runs");
            if !test_run_path.try_exists().is_ok_and(|b| b) {
                if let Err(e) = fs::DirBuilder::new().create(test_run_path) {
                    return Err(ProgramError::new(format!(
                        "Failed to create test_runs directory '{}'. ",
                        e.kind()
                    )));
                }
            }
        }

        if let Err(e) = fs::DirBuilder::new().create(&proj_name) {
            return Err(ProgramError::new(format!(
                "Failed to create project folder '{}'. ",
                e.kind()
            )));
        }

        // create terminal
        let proj_dir = env::current_dir().unwrap().join(&proj_name);
        let mut terminal = Terminal::new(proj_dir);

        // setup venv
        terminal.run_cmd(
            "python -m venv env",
            "Failed to create virtual env.",
            "setting up virtual environment",
            flags,
        )?;

        let activate_cmd = if cfg!(windows) {
            "env\\Scripts\\activate.bat"
        } else {
            "source env/bin/activate"
        };

        // install django
        terminal.run_cmd(
            &format!("{activate_cmd} && pip install django"),
            "Failed to install django with pip.",
            "installing django",
            flags,
        )?;

        // start a django project
        terminal.run_cmd(
            &format!("{activate_cmd} && django-admin startproject core ."),
            "Failed to start a django project.",
            "starting a django project",
            flags,
        )?;

        // run the dev server
        terminal.run_cmd(
            &format!("{activate_cmd} && python manage.py runserver"),
            "Failed to run dev server.",
            "running dev server...",
            flags,
        )?;

        // TODO open it in file explorer/code

        Ok(())
    }

    fn set_up_react_project(&self, flags: &[Flag]) -> PEResult {
        // create dir
        let flag_set_proj_name = Flag::get_project_name(&flags);
        let mut proj_name = if let Some(s) = flag_set_proj_name {
            s
        } else {
            prompt_input("Enter project name: ")?
        };

        proj_name = proj_name.trim().to_string();
        Flag::log_if_verbose(format!("creating {proj_name:?} directory").as_str(), flags);

        let is_test_run = Flag::is_test_run(&flags);
        if is_test_run {
            proj_name = format!("test_runs/{proj_name}");
            let test_run_path = Path::new("test_runs");
            if !test_run_path.try_exists().is_ok_and(|b| b) {
                if let Err(e) = fs::DirBuilder::new().create(test_run_path) {
                    return Err(ProgramError::new(format!(
                        "Failed to create test_runs directory '{}'. ",
                        e.kind()
                    )));
                }
            }
        }

        if let Err(e) = fs::DirBuilder::new().create(&proj_name) {
            return Err(ProgramError::new(format!(
                "Failed to create project folder '{}'. ",
                e.kind()
            )));
        }

        // create terminal
        let proj_dir = env::current_dir().unwrap().join(&proj_name);
        let mut terminal = Terminal::new(proj_dir.clone());

        // run vite cli
        terminal.run_cmd(
            "npm create vite@latest",
            "Failed to create vite app with npm.",
            "creating vite app",
            flags,
        )?;

        // cd into project
        let proj_dir_contents = proj_dir.read_dir();
        if let Ok(mut dirs) = proj_dir_contents {
            if let Some(dir) = dirs.next() {
                if dir.is_ok() {
                    terminal.working_dir = dir.unwrap().path();
                    green_log(format!("moved into: {:#?}", terminal.working_dir).as_str());
                };
            };
        };

        // npm install
        terminal.run_cmd(
            "npm install",
            "Failed to install node modules.",
            "installing node modules...",
            flags,
        )?;

        // run the dev server
        terminal.run_cmd(
            "npm run dev",
            "Failed to run dev server.",
            "running dev server...",
            flags,
        )?;

        // TODO open it in file explorer/code

        Ok(())
    }

    fn set_up_next_project(&self) -> PEResult {
        todo!("set_up_next_project")
    }
}

impl Terminal {
    pub fn new(working_dir: PathBuf) -> Self {
        if cfg!(windows) {
            Self {
                base_shell_args: [String::from("cmd"), String::from("/C")],
                working_dir,
            }
        } else {
            Self {
                base_shell_args: [String::from("sh"), String::from("-c")],
                working_dir,
            }
        }
    }

    pub fn run_cmd(&mut self, cmd: &str, err_msg: &str, log_msg: &str, flags: &[Flag]) -> PEResult {
        Flag::log_if_verbose(log_msg, flags);
        let output = Command::new(&self.base_shell_args[0])
            .arg(&self.base_shell_args[1])
            .arg(cmd)
            .current_dir(&self.working_dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output();

        if let Err(e) = output {
            return Err(ProgramError::new(format!("{err_msg} {e}")));
        } else {
            let output = output.unwrap();
            let output_text = String::from_utf8_lossy(&output.stdout);
            yellow_log(&output_text);

            if !output.status.success() {
                return Err(ProgramError::new(err_msg.to_string()));
            }
        }

        Ok(())
    }
}

impl Flag {
    pub fn log_if_verbose(msg: &str, flags: &[Self]) {
        if flags.contains(&Self::Verbose) {
            blue_log(msg);
        }
    }

    fn get_project_name(flags: &[Self]) -> Option<String> {
        let name = flags.iter().find(|flag| match flag {
            Self::Name(_) => true,
            _ => false,
        });

        if let Some(Self::Name(Value(Some(name)))) = name {
            Some(name.to_string())
        } else {
            None
        }
    }

    fn is_test_run(flags: &[Self]) -> bool {
        flags.contains(&Self::Test)
    }

    pub fn handle_help_flag(prog_args: &ProgramArguments) -> DidSomething {
        if prog_args.get_flags().contains(&Self::Help) {
            if let Some(project_type) = prog_args.get_project_type() {
                let proj_option = VALID_PROJECT_OPTIONS
                    .iter()
                    .find(|opt| opt.1 == *project_type);

                if let Some((_, _, description)) = proj_option {
                    println!(
                        "PROJECT TYPE: {}\n\n{description}\n\n{}\n{}\n\n",
                        format!("{project_type:?}").blue(),
                        "Flags".blue(),
                        VALID_FLAGS
                            .iter()
                            .enumerate()
                            .map(|(index, opt)| format!(
                                "{}. {} | {}: {}",
                                index.to_string().blue(),
                                opt.0.green(),
                                opt.1.green(),
                                opt.3
                            ))
                            .reduce(|acc_str, s| format!("{acc_str}\n{s}"))
                            .unwrap_or("".to_string())
                    );
                } else {
                    println!("No help text found for project type: {project_type:?}")
                };
            } else {
                println!(
                    "{CLI_HELP_TEXT_WITHOUT_PROJECT_NOR_FLAG_OPTION_DESCRIPTIONS}\n\n{}:\n{}\n\n{}:\n{}\n\n",
                     "Project Types".blue(),
                    VALID_PROJECT_OPTIONS
                    .iter()
                    .enumerate()
                    .map(|(index, opt)| format!("{}. {}: {}", index.to_string().blue(), opt.0.green(), opt.2) )
                    .reduce(|acc_str, s| format!("{acc_str}\n{s}")).unwrap_or("".to_string())
                    ,"Flags".blue(),
                    VALID_FLAGS
                    .iter()
                    .enumerate()
                    .map(|(index, opt)| format!("{}. {} | {}: {}", index.to_string().blue(), opt.0.green(), opt.1.green(), opt.3) )
                    .reduce(|acc_str, s| format!("{acc_str}\n{s}")).unwrap_or("".to_string())

                );
            }

            DidSomething::Yes
        } else {
            DidSomething::No
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::VALID_PROJECT_OPTIONS;

    #[test]
    fn valid_raw_args_return_bin_args() {
        for valid_option in VALID_PROJECT_OPTIONS {
            let raw_args = [valid_option.0].into_iter().map(|s_ref| s_ref.to_string());
            ProgramArguments::build(raw_args).unwrap();
        }
    }

    #[test]
    #[should_panic]
    fn invalid_raw_args_return_error() {
        let invalid_raw_args = [String::from("invalid-option")].into_iter();
        ProgramArguments::build(invalid_raw_args).unwrap();
    }
}
