use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::{
    constants::{VALID_FLAGS, VALID_PROJECT_OPTIONS},
    utils::{
        self, log_if_verbose, prompt_input, run_child_cmd, run_seperate_cmd, yellow_log, PEResult,
    },
};

#[derive(Debug)]
pub struct ProgramError {
    message: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ProjectType {
    Django,
    React,
    Next,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Flag {
    Help,
    Verbose,
}

struct Terminal {
    working_dir: PathBuf,
    base_shell_args: [String; 2],
}

pub struct ProgramArguments {
    project_type: ProjectType,
    flags: Vec<Flag>,
}

impl ProgramArguments {
    pub fn build<T: Iterator<Item = String>>(mut raw_args: T) -> PEResult<Self> {
        let mut project_type: Option<ProjectType> = None;
        let mut flags: Vec<Flag> = vec![];

        while let Some(mut arg) = raw_args.next() {
            arg = arg.trim().to_lowercase();
            if arg.starts_with("-") {
                flags.push(Self::map_string_to_flag(&arg)?);
            } else if project_type.is_none() {
                project_type = Some(Self::map_string_to_project_type(&arg)?);
            } else {
                return Err(ProgramError::new(format!(
                    "You can only provide one project type! Found extra type '{arg}'",
                )));
            }
        }

        if project_type.is_some() {
            Ok(Self {
                project_type: project_type.unwrap(),
                flags,
            })
        } else {
            Err(ProgramError::new(
                "No valid project type provided".to_string(),
            ))
        }
    }

    pub fn get_project_type(&self) -> &ProjectType {
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

    fn map_string_to_flag(s: &str) -> PEResult<Flag> {
        let flag = VALID_FLAGS.iter().find(|flag| flag.0 == s || flag.1 == s);

        if let Some(flag) = flag {
            Ok(flag.2)
        } else {
            Err(ProgramError::new(format!(
                "'{s}' is not a valid flag, run again with --help or -h for more info."
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
        log_if_verbose(format!("setting up {self:?} project").as_str(), flags);

        match self {
            ProjectType::Django => self.set_up_django_project(flags),
            ProjectType::React => self.set_up_react_project(),
            ProjectType::Next => self.set_up_next_project(),
        }
    }

    pub fn check_for_required_tooling(&self, flags: &[Flag]) -> PEResult {
        log_if_verbose(
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
        // check for any of node, deno, bun
        let cmds = ["node --version || deno --version || bun --version"];
        if utils::check_if_any_command_passes(&cmds).is_err() {
            return Err(ProgramError::new(format!(
                "Could not confirm if any of Node, Deno, or Bun is installed, in order to set up a {self:?} project."
            )));
        }

        // check for any of npm, yarn, pnpm, deno, bun
        let cmds = [
            "npm --version || yarn --version || pnpm --version || bun --version || deno --version",
        ];
        if utils::check_if_any_command_passes(&cmds).is_err() {
            return Err(ProgramError::new(format!(
                "Could not confirm if any of Npm, Yarn, Pnpm, Deno, or Bun is installed, in order to set up a {self:?} project."
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

    fn _set_up_django_project_old(&self, flags: &[Flag]) -> PEResult {
        // create dir
        let mut p_name = prompt_input("Enter project name: ")?;
        p_name = p_name.trim().to_string();

        log_if_verbose(format!("creating {p_name:?} directory").as_str(), flags);

        if let Err(e) = fs::DirBuilder::new().create(&p_name) {
            return Err(ProgramError::new(format!(
                "Failed to create project folder '{}'. ",
                e.kind()
            )));
        }

        log_if_verbose(format!("moving into {p_name:?} directory").as_str(), flags);

        // run in a single process
        {
            if let Err(e) = run_child_cmd(format!("cd {p_name:?}").as_str()) {
                return Err(ProgramError::new(format!(
                    "Failed to change into project dir. {}",
                    e.msg()
                )));
            }

            // setup venv
            log_if_verbose(format!("setting up virtual environment").as_str(), flags);

            if run_child_cmd("python -m venv env").is_err() {
                if let Err(e) = run_child_cmd("python3 -m venv env") {
                    return Err(ProgramError::new(format!(
                        "Failed to create virtual env. {}",
                        e.msg()
                    )));
                }
            }

            // activate venv
            log_if_verbose(format!("activating...").as_str(), flags);

            if run_seperate_cmd(".env/scripts/activate").is_err() {
                if let Err(e) = run_seperate_cmd("source env\\bin\\scripts\\activate") {
                    return Err(ProgramError::new(format!(
                        "Failed to activate virtual env. {}",
                        e.msg()
                    )));
                }
            }

            // install and start a django project
            log_if_verbose(
                format!("installing and starting a django project").as_str(),
                flags,
            );

            if run_child_cmd("pip install django").is_err() {
                if run_child_cmd("python -m pip install django").is_err() {
                    if let Err(e) = run_child_cmd("python3 -m pip install django") {
                        return Err(ProgramError::new(format!(
                            "Failed to install django with pip. {}",
                            e.msg()
                        )));
                    }
                }
            }

            if let Err(e) = run_child_cmd("django-admin startproject core") {
                return Err(ProgramError::new(format!(
                    "Failed to 'django startproject'. {}",
                    e.msg()
                )));
            }

            // run the dev server
            log_if_verbose(format!("running dev server...").as_str(), flags);

            if run_child_cmd("python manage.py runserver").is_err() {
                if let Err(e) = run_child_cmd("python3 manage.py runserver") {
                    return Err(ProgramError::new(format!(
                        "Failed to run dev server. {}",
                        e.msg()
                    )));
                }
            }

            // TODO open it in file explorer/code
        }

        Ok(())
    }

    fn set_up_django_project(&self, flags: &[Flag]) -> PEResult {
        // create dir
        let mut proj_name = prompt_input("Enter project name: ")?;
        proj_name = proj_name.trim().to_string();
        log_if_verbose(format!("creating {proj_name:?} directory").as_str(), flags);
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

    fn set_up_react_project(&self) -> PEResult {
        todo!("set_up_react_project")
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
        log_if_verbose(log_msg, flags);
        let output = Command::new(&self.base_shell_args[0])
            .arg(&self.base_shell_args[1])
            .arg(cmd)
            .current_dir(&self.working_dir)
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
