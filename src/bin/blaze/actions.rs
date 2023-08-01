use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use blaze::dependencies::{get_latest_version, Dependencies};
use colored::Colorize;
use inquire::Text;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use crate::cli::Command;
use crate::error;

impl Command {
    pub async fn run(&self) {
        match self {
            Command::Install { package_names } => {
                let mut dependencies;

                if !package_names.is_empty() {
                    dependencies = Dependencies::default();

                    for package_name in package_names {
                        let version = match get_latest_version(package_name).await {
                            Ok(version) => version,
                            Err(error) => {
                                error::print_error(&error);
                                return;
                            }
                        };

                        dependencies
                            .dependencies
                            .insert(package_name.to_string(), version);

                        if let Err(e) = dependencies.write_dependencies_to_package_json() {
                            error::print_error(&e.to_string());
                            return;
                        }
                    }
                } else {
                    dependencies = match Dependencies::from_package_json() {
                        Ok(dependencies) => dependencies,
                        Err(error) => {
                            error::print_error(&error.to_string());
                            return;
                        }
                    };
                }

                if let Err(e) = dependencies.download_dependencies().await {
                    error::print_error(&e);
                };
            }
            Command::Init {} => {
                let project_name = match Text::new("Project Name")
                    .with_default("my-amazing-project")
                    .prompt()
                {
                    Ok(name) => name,
                    Err(_) => {
                        error::print_error(
                            "An unexpected error occurred while getting project name",
                        );
                        return;
                    }
                };

                let project_description = match Text::new("Description")
                    .with_default("My amazing project")
                    .with_help_message("A short description of your project")
                    .prompt()
                {
                    Ok(description) => description,
                    Err(_) => {
                        error::print_error(
                            "An unexpected error occurred while getting project description",
                        );
                        return;
                    }
                };

                let project_version = match Text::new("Version")
                    .with_default("1.0.0")
                    .with_help_message("The initial version of your project")
                    .prompt()
                {
                    Ok(version) => version,
                    Err(_) => {
                        error::print_error(
                            "An unexpected error occurred while getting project version",
                        );
                        return;
                    }
                };

                let test_command = match Text::new("Test Command")
                    .with_default("echo \"Error: no test specified\" && exit 1")
                    .prompt()
                {
                    Ok(command) => command,
                    Err(_) => {
                        error::print_error(
                            "An unexpected error occurred while getting test command",
                        );
                        return;
                    }
                };

                let project_repository = match Text::new("Repository")
                    .with_default("")
                    .with_help_message("The repository of your project")
                    .prompt()
                {
                    Ok(repository) => repository,
                    Err(_) => {
                        error::print_error(
                            "An unexpected error occurred while getting project repository",
                        );
                        return;
                    }
                };

                let project_keywords = match Text::new("Keywords").with_default("").prompt() {
                    Ok(keywords) => keywords,
                    Err(_) => {
                        error::print_error(
                            "An unexpected error occurred while getting project keywords",
                        );
                        return;
                    }
                };

                let project_author = match Text::new("Author")
                    .with_default("John Doe")
                    .with_help_message("The author of your project")
                    .prompt()
                {
                    Ok(author) => author,
                    Err(_) => {
                        error::print_error(
                            "An unexpected error occurred while getting project author",
                        );
                        return;
                    }
                };

                #[derive(Deserialize, Serialize, Debug)]
                struct PackageJson {
                    name: String,
                    version: String,
                    description: String,
                    author: String,
                    repository: String,
                    license: String,
                    keywords: String,
                    main: String,
                    scripts: Scripts,
                    dependencies: PackageJsonDependencies,
                }

                #[derive(Deserialize, Serialize, Debug)]
                struct Scripts {
                    test: String,
                }

                #[derive(Deserialize, Serialize, Debug)]
                struct PackageJsonDependencies {
                    dependencies: HashMap<String, String>,
                }

                let project_license = match Text::new("License")
                    .with_default("MIT")
                    .with_help_message("The license of your project")
                    .prompt()
                {
                    Ok(license) => license,
                    Err(_) => "MIT".to_string(),
                };

                let package_json = PackageJson {
                    name: project_name,
                    version: project_version,
                    description: project_description,
                    author: project_author,
                    repository: project_repository,
                    license: project_license,
                    keywords: project_keywords,
                    main: "index.js".to_string(),
                    scripts: Scripts { test: test_command },
                    dependencies: PackageJsonDependencies {
                        dependencies: HashMap::new(),
                    },
                };

                let json_text = match to_string_pretty(&package_json) {
                    Ok(json) => json,
                    Err(e) => {
                        println!("Error converting to JSON: {:?}", e);
                        return;
                    }
                };

                if let Err(e) = fs::create_dir_all(&package_json.name) {
                    error::print_error(&e.to_string());
                    return;
                };

                let mut file =
                    match File::create(Path::new(&package_json.name).join("package.json")) {
                        Ok(file) => file,
                        Err(e) => {
                            error::print_error(&e.to_string());
                            return;
                        }
                    };

                if let Err(e) = file.write_all(json_text.as_bytes()) {
                    error::print_error(&e.to_string());
                }
            }

            Command::Version { verbose } => {
                if verbose.is_some() && verbose.unwrap() {
                    println!("{} {}", "Version".green(), env!("BUILD_VERSION"));
                    println!("{} {}", "Build Timestamp".green(), env!("BUILD_TIMESTAMP"));
                    println!("{} {}", "Author".green(), env!("CARGO_PKG_AUTHORS"));
                    println!("{} {}", "License".green(), env!("CARGO_PKG_LICENSE"));
                    println!("{} {}", "Repository".green(), env!("CARGO_PKG_REPOSITORY"));
                } else {
                    println!("{} {}", "Version".green(), env!("BUILD_VERSION"));
                }
            }
        }
    }
}
