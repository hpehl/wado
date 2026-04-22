use crate::wildfly::{
    AdminContainer, DEFAULT_SERVER_OFFSET, Server, ServerType, StartSpec, apply_offsets,
};
use anyhow::bail;
use clap::ArgMatches;
use fs::read_to_string;
use std::fs;
use std::path::Path;
use wildfly_container_versions::WildFlyContainer;

// ------------------------------------------------------ sorted a-z

pub fn admin_containers_argument(matches: &ArgMatches) -> Vec<AdminContainer> {
    let standalone = matches.get_flag("standalone");
    let domain = matches.get_flag("domain");
    versions_argument(matches)
        .iter()
        .flat_map(|wc| {
            if standalone {
                vec![AdminContainer::new(wc.clone(), ServerType::Standalone)]
            } else if domain {
                AdminContainer::domain(wc.clone())
            } else {
                AdminContainer::all_types(wc.clone())
            }
        })
        .collect::<Vec<_>>()
}

pub fn name_argument<F>(name: &str, matches: &ArgMatches, f: F) -> String
where
    F: FnOnce() -> String,
{
    matches
        .get_one::<String>(name)
        .map(|s| s.to_string())
        .unwrap_or_else(f)
}

pub fn operations_argument(matches: &ArgMatches) -> Vec<String> {
    let mut operations = matches
        .get_many::<String>("operations")
        .unwrap_or_default()
        .flat_map(|operation| {
            operation
                .split(',')
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .filter(|op| {
            if is_valid_cli_operation(op) {
                true
            } else {
                eprintln!(
                    "Skipping invalid CLI operation (must start with '/' or ':'): {}",
                    op
                );
                false
            }
        })
        .collect::<Vec<_>>();
    if matches.contains_id("cli")
        && let Some(cli_path) = matches.get_one::<String>("cli")
    {
        let path = Path::new(cli_path);
        match path.canonicalize().and_then(|_| read_to_string(path)) {
            Ok(content) => {
                for line in content.lines().map(|s| s.trim().to_string()) {
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if is_valid_cli_operation(&line) {
                        operations.push(line);
                    } else {
                        eprintln!(
                            "Skipping invalid CLI operation (must start with '/' or ':'): {}",
                            line
                        );
                    }
                }
            }
            Err(err) => {
                eprintln!("Failed to read file {}: {}", cli_path, err);
            }
        }
    }
    operations
}

pub fn parameters_argument(matches: &ArgMatches) -> Vec<String> {
    matches
        .get_many::<String>("wildfly-parameters")
        .unwrap_or_default()
        .cloned()
        .collect::<Vec<_>>()
}

pub fn start_spec(
    matches: &ArgMatches,
    wildfly_container: &WildFlyContainer,
    server_type: ServerType,
) -> StartSpec {
    let admin_container = AdminContainer::new(wildfly_container.clone(), server_type);
    let offset = matches.get_one::<u16>("offset").copied().unwrap_or(0);
    let has_offset = offset > 0;
    let custom_http = matches
        .get_one::<u16>("http")
        .map(|p| p + offset)
        .or_else(|| {
            if has_offset {
                Some(wildfly_container.http_port() + offset)
            } else {
                None
            }
        });
    let custom_management = matches
        .get_one::<u16>("management")
        .map(|p| p + offset)
        .or_else(|| {
            if has_offset {
                Some(wildfly_container.management_port() + offset)
            } else {
                None
            }
        });
    StartSpec {
        admin_container,
        custom_name: matches.get_one::<String>("name").cloned(),
        custom_http,
        custom_management,
    }
}

pub fn server_argument(matches: &ArgMatches) -> Vec<Server> {
    let servers = matches
        .get_many::<Vec<Server>>("server")
        .unwrap_or_default()
        .cloned()
        .collect::<Vec<_>>();
    let servers = servers
        .iter()
        .flat_map(|server| server.clone())
        .collect::<Vec<_>>();
    apply_offsets(servers, DEFAULT_SERVER_OFFSET)
}

pub fn username_password_argument(matches: &ArgMatches) -> (&str, &str) {
    let username = matches
        .get_one::<String>("username")
        .expect("No username given")
        .as_str();
    let password = matches
        .get_one::<String>("password")
        .expect("No password given")
        .as_str();
    (username, password)
}

pub fn versions_argument(matches: &ArgMatches) -> Vec<WildFlyContainer> {
    matches
        .get_one::<Vec<WildFlyContainer>>("wildfly-version")
        .expect("Argument <wildfly-version> expected!")
        .clone()
}

// ------------------------------------------------------ validation

pub fn validate_multiple_versions(matches: &ArgMatches, options: &[&str]) -> anyhow::Result<()> {
    for option in options {
        if matches.contains_id(option) {
            bail!(
                "Option <{}> is not allowed when multiple <wildfly-version> are specified!",
                option
            );
        }
    }
    Ok(())
}

pub fn extract_config(parameters: &[String], default: &str) -> String {
    let mut iter = parameters.iter();
    while let Some(param) = iter.next() {
        if param == "-c" {
            if let Some(value) = iter.next() {
                return value.clone();
            }
        } else if let Some(value) = param.strip_prefix("--server-config=") {
            return value.to_string();
        }
    }
    default.to_string()
}

// ------------------------------------------------------ validation

fn is_valid_cli_operation(operation: &str) -> bool {
    let trimmed = operation.trim();
    trimmed.starts_with('/') || trimmed.starts_with(':')
}
