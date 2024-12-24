use crate::data::{Flag, ProjectType};

/// The only project options you can pass to the CLI, along with their corresponding ProjectType enums, and descriptions.
/// `option = (option, ProjectType, description)`.
pub const VALID_PROJECT_OPTIONS: [(&str, ProjectType, &str); 3] = [
    ("django", ProjectType::Django, "Python Django web-framework project. Requires Python version 3. Sets up a virtual environment 'venv' using the standard venv module; Installs Django into venv using pip; Starts a Django project 'core'; Runs the Django dev server."), 
    ("react", ProjectType::React,"Javascript (or TS) React web-app project with Vite. Currently requires/uses Node.js. Uses NPM and Vite CLI to set up a React project with further configurations prompted to user (piped from Vite CLI). Runs the Vite dev server"), 
    ("next", ProjectType::Next,"Javascript (or TS) Next web-framework project. Currently requires/uses Node.js. Uses NPM and Next CLI to set up a Next project with further configurations prompted to user (piped from Next CLI). Runs the Next dev server")
    ];

/// The only flags you can pass to the CLI, along with their short forms, corresponding Flag enums, and descriptions.
/// Some flags only have an effect when passed with certain options. In these cases other non compatible flags will be completely egnored.
/// `flag = (long_form, short_form, Flag, description)`.
pub const VALID_FLAGS: [(&str, &str, Flag, &str); 2] = [
    ("--help", "-h", Flag::Help, "Show CLI help. If passed with an option, shows option description and optional flags with their descriptions."),
    ("--verbose", "-v",Flag::Verbose, "Show all CLI output."),
];
