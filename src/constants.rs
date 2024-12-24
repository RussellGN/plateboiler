/// The only project options you can pass to the CLI, along with their descriptions.
/// `option = (option, description)`.
pub const VALID_OPTIONS: [(&str, &str); 3] = [
    ("django", "Python Django web-framework project. Requires Python version 3. Sets up a virtual environment 'venv' using the standard venv module; Installs Django into venv using pip; Starts a Django project 'core'; Runs the Django dev server."), 
    ("react", "Javascript (or TS) React web-app project with Vite. Currently requires/uses Node.js. Uses NPM and Vite CLI to set up a React project with further configurations prompted to user (piped from Vite CLI). Runs the Vite dev server"), 
    ("next", "Javascript (or TS) Next web-framework project. Currently requires/uses Node.js. Uses NPM and Next CLI to set up a Next project with further configurations prompted to user (piped from Next CLI). Runs the Next dev server")
    ];

/// The only flags you can pass to the CLI, along with their short forms and descriptions.
/// Some flags only have an effect when passed with certain options. In these cases other non compatible flags will be completely egnored.
/// `flag = (long_form, short_form, description)`.
pub const VALID_FLAGS: [(&str, &str, &str); 2] = [
    ("--help", "-h", "Show CLI help. If passed with an option, shows option description and optional flags with their descriptions."),
    ("--verbose", "-v", "Show all CLI output."),
];
