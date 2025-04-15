use crate::wildfly::{AdminContainer, Ports, Server};
use clap::ArgMatches;
use fs::read_to_string;
use std::fs;
use wildfly_container_versions::WildFlyContainer;

// ------------------------------------------------------ sorted a-z

pub fn admin_containers_argument(matches: &ArgMatches) -> Vec<AdminContainer> {
    let standalone = matches.get_flag("standalone");
    let domain = matches.get_flag("domain");
    let wildfly_containers = versions_argument(matches);
    let admin_containers = wildfly_containers
        .iter()
        .flat_map(|wc| {
            if standalone {
                vec![AdminContainer::standalone(wc.clone())]
            } else if domain {
                AdminContainer::domain(wc.clone())
            } else {
                AdminContainer::all_types(wc.clone())
            }
        })
        .collect::<Vec<_>>();
    admin_containers
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
        .collect::<Vec<_>>();
    if matches.contains_id("cli") {
        if let Some(cli_path) = matches.get_one::<String>("cli") {
            match read_to_string(cli_path) {
                Ok(content) => {
                    operations.extend(content.lines().map(|s| s.trim().to_string()));
                }
                Err(err) => {
                    eprintln!("Failed to read file {}: {}", cli_path, err);
                }
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

pub fn port_argument(matches: &ArgMatches, wildfly_container: &WildFlyContainer) -> Ports {
    let offset = matches.get_one::<u16>("offset").unwrap_or(&0);
    let http = matches
        .get_one::<u16>("http")
        .unwrap_or(&wildfly_container.http_port())
        + offset;
    let management = matches
        .get_one::<u16>("management")
        .unwrap_or(&wildfly_container.management_port())
        + offset;
    Ports { http, management }
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
    apply_offsets(servers, 100)
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

// ------------------------------------------------------ helpers

fn apply_offsets(servers: Vec<Server>, offset: u16) -> Vec<Server> {
    if servers.len() > 1 {
        let mut last_offset = 0;
        servers
            .iter()
            .enumerate()
            .map(|(index, server)| {
                if index == 0 {
                    server.clone()
                } else {
                    let server_with_offset = if server.offset == 0 {
                        server.with_offset(last_offset + offset)
                    } else {
                        server.clone()
                    };
                    last_offset = server_with_offset.offset;
                    server_with_offset
                }
            })
            .collect::<Vec<_>>()
    } else {
        servers
    }
}

// ------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_offsets_empty() {
        let servers = vec![];
        assert_eq!(apply_offsets(servers, 100), vec![]);
    }

    #[test]
    fn apply_offsets_single_server() {
        let server = Server::parse_server("server1").unwrap();
        let input = vec![server.clone()];
        let expected = vec![server.clone()];
        assert_eq!(apply_offsets(input, 100), expected);
    }

    #[test]
    fn apply_offsets_multiple_servers() {
        let server0 = Server::parse_server("server0").unwrap();
        let server1 = Server::parse_server("server1").unwrap();
        let server2 = Server::parse_server("server2").unwrap();
        let server3 = Server::parse_server("server3").unwrap();
        let input = vec![
            server0.clone(),
            server1.clone(),
            server2.clone(),
            server3.clone(),
        ];
        let expected = vec![
            server0.clone(),
            server1.with_offset(100),
            server2.with_offset(200),
            server3.with_offset(300),
        ];
        assert_eq!(apply_offsets(input, 100), expected);
    }

    #[test]
    fn apply_offsets_multiple_servers_custom_offset() {
        let server0 = Server::parse_server("server0").unwrap();
        let server1 = Server::parse_server("server1").unwrap();
        let server2 = Server::parse_server("server2:50").unwrap();
        let server3 = Server::parse_server("server3").unwrap();
        let input = vec![
            server0.clone(),
            server1.clone(),
            server2.clone(),
            server3.clone(),
        ];
        let expected = vec![
            server0.clone(),
            server1.with_offset(100),
            server2.with_offset(50),
            server3.with_offset(150),
        ];
        assert_eq!(apply_offsets(input, 100), expected);
    }
}
