mod app;
mod args;
mod build;
mod cli;
mod command;
mod console;
mod constants;
mod dc;
mod hc;
mod podman;
mod progress;
mod ps;
mod push;
mod resources;
mod standalone;
mod topology;
mod wildfly;

use crate::build::build;
use crate::cli::cli;
use crate::console::console;
use crate::dc::{dc_start, dc_stop};
use crate::ps::ps;
use crate::push::push;
use crate::standalone::{standalone_start, standalone_stop};
use crate::topology::{topology_start, topology_stop};
use crate::wildfly::Server;
use anyhow::Result;
use app::build_app;
use clap::value_parser;
use hc::{hc_start, hc_stop};
use std::path::PathBuf;
use wildfly_container_versions::WildFlyContainer;

//noinspection DuplicatedCode
#[tokio::main]
async fn main() -> Result<()> {
    let matches = build_app()
        .mut_subcommand("build", |sub_cmd| {
            sub_cmd.mut_arg("wildfly-version", |arg| {
                arg.value_parser(parse_version_enumeration)
            })
        })
        .mut_subcommand("push", |sub_cmd| {
            sub_cmd.mut_arg("wildfly-version", |arg| {
                arg.value_parser(parse_version_enumeration)
            })
        })
        .mut_subcommand("start", |sub_cmd| {
            sub_cmd.mut_arg("wildfly-version", |arg| {
                arg.value_parser(parse_version_enumeration)
            })
        })
        .mut_subcommand("stop", |sub_cmd| {
            sub_cmd.mut_arg("wildfly-version", |arg| {
                arg.value_parser(parse_version_enumeration)
            })
        })
        .mut_subcommand("dc", |sub_cmd| {
            sub_cmd.mut_subcommand("start", |sub_sub_cmd| {
                sub_sub_cmd
                    .mut_arg("wildfly-version", |arg| {
                        arg.value_parser(parse_version_enumeration)
                    })
                    .mut_arg("server", |arg| arg.value_parser(parse_servers))
            })
        })
        .mut_subcommand("dc", |sub_cmd| {
            sub_cmd.mut_subcommand("stop", |sub_sub_cmd| {
                sub_sub_cmd.mut_arg("wildfly-version", |arg| {
                    arg.value_parser(parse_version_enumeration)
                })
            })
        })
        .mut_subcommand("hc", |sub_cmd| {
            sub_cmd.mut_subcommand("start", |sub_sub_cmd| {
                sub_sub_cmd
                    .mut_arg("wildfly-version", |arg| {
                        arg.value_parser(parse_version_enumeration)
                    })
                    .mut_arg("server", |arg| arg.value_parser(parse_servers))
            })
        })
        .mut_subcommand("hc", |sub_cmd| {
            sub_cmd.mut_subcommand("stop", |sub_sub_cmd| {
                sub_sub_cmd.mut_arg("wildfly-version", |arg| {
                    arg.value_parser(parse_version_enumeration)
                })
            })
        })
        .mut_subcommand("topology", |sub_cmd| {
            sub_cmd.mut_subcommand("start", |sub_sub_cmd| {
                sub_sub_cmd.mut_arg("setup", |arg| arg.value_parser(value_parser!(PathBuf)))
            })
        })
        .mut_subcommand("topology", |sub_cmd| {
            sub_cmd.mut_subcommand("stop", |sub_sub_cmd| {
                sub_sub_cmd.mut_arg("setup", |arg| arg.value_parser(value_parser!(PathBuf)))
            })
        })
        .mut_subcommand("console", |sub_cmd| {
            sub_cmd.mut_arg("wildfly-version", |arg| {
                arg.value_parser(parse_version_enumeration)
            })
        })
        .mut_subcommand("cli", |sub_cmd| {
            sub_cmd.mut_arg("wildfly-version", |arg| arg.value_parser(parse_version))
        })
        .get_matches();

    match matches.subcommand() {
        Some(("build", m)) => build(m),
        Some(("push", m)) => push(m),

        Some(("start", m)) => standalone_start(m),
        Some(("stop", m)) => standalone_stop(m),

        Some(("dc", sub_matches)) => match sub_matches.subcommand() {
            Some(("start", m)) => dc_start(m),
            Some(("stop", m)) => dc_stop(m),
            _ => unreachable!("Unknown subcommand"),
        },

        Some(("hc", sub_matches)) => match sub_matches.subcommand() {
            Some(("start", m)) => hc_start(m),
            Some(("stop", m)) => hc_stop(m),
            _ => unreachable!("Unknown subcommand"),
        },

        Some(("topology", sub_matches)) => match sub_matches.subcommand() {
            Some(("start", m)) => topology_start(m),
            Some(("stop", m)) => topology_stop(m),
            _ => unreachable!("Unknown subcommand"),
        },

        Some(("ps", _)) => ps(),
        Some(("console", m)) => console(m),
        Some(("cli", m)) => cli(m),

        _ => unreachable!("Unknown subcommand"),
    }?;
    Ok(())
}

// ------------------------------------------------------ validation

fn parse_version_enumeration(range: &str) -> Result<Vec<WildFlyContainer>, String> {
    WildFlyContainer::enumeration(range).map_err(|err| err.to_string())
}

fn parse_version(version: &str) -> Result<WildFlyContainer, String> {
    WildFlyContainer::version(version).map_err(|e| e.to_string())
}

fn parse_servers(server: &str) -> Result<Vec<Server>, String> {
    Server::parse_servers(server).map_err(|err| err.to_string())
}
