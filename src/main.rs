mod app;
mod args;
mod build;
mod cli;
mod completions;
mod console;
mod constants;
mod container;
mod dc;
mod hc;
mod image;
mod label;
mod progress;
mod ps;
mod push;
mod resources;
mod standalone;
mod topology;
mod wildfly;
mod wildfly_version;

use crate::build::build;
use crate::cli::cli;
use crate::completions::completions;
use crate::console::console;
use crate::container::{
    complete_running_names, complete_running_topologies, complete_running_versions,
};
use crate::dc::{dc_start, dc_stop};
use crate::image::images;
use crate::ps::ps;
use crate::push::push;
use crate::standalone::{standalone_start, standalone_stop};
use crate::topology::{topology_start, topology_stop};
use crate::wildfly::Server;
use crate::wildfly::ServerType::{DomainController, HostController, Standalone};
use crate::wildfly_version::complete_versions;
use anyhow::Result;
use app::build_app;
use clap::value_parser;
use clap_complete::engine::ArgValueCompleter;
use hc::{hc_start, hc_stop};
use std::path::PathBuf;
use wildfly_container_versions::WildFlyContainer;

fn build_app_full() -> clap::Command {
    build_app()
        .mut_subcommand("build", |sub_cmd| {
            sub_cmd.mut_arg("wildfly-version", |arg| {
                arg.value_parser(parse_version_enumeration)
                    .add(ArgValueCompleter::new(complete_versions))
            })
        })
        .mut_subcommand("push", |sub_cmd| {
            sub_cmd.mut_arg("wildfly-version", |arg| {
                arg.value_parser(parse_version_enumeration)
                    .add(ArgValueCompleter::new(complete_versions))
            })
        })
        .mut_subcommand("start", |sub_cmd| {
            sub_cmd.mut_arg("wildfly-version", |arg| {
                arg.value_parser(parse_version_enumeration)
                    .add(ArgValueCompleter::new(complete_versions))
            })
        })
        .mut_subcommand("stop", |sub_cmd| {
            sub_cmd
                .mut_arg("wildfly-version", |arg| {
                    arg.value_parser(parse_version_enumeration)
                        .add(ArgValueCompleter::new(complete_running_versions(vec![
                            Standalone,
                        ])))
                })
                .mut_arg("name", |arg| {
                    arg.add(ArgValueCompleter::new(complete_running_names(vec![
                        Standalone,
                    ])))
                })
        })
        .mut_subcommand("dc", |sub_cmd| {
            sub_cmd.mut_subcommand("start", |sub_sub_cmd| {
                sub_sub_cmd
                    .mut_arg("wildfly-version", |arg| {
                        arg.value_parser(parse_version_enumeration)
                            .add(ArgValueCompleter::new(complete_versions))
                    })
                    .mut_arg("server", |arg| arg.value_parser(parse_servers))
            })
        })
        .mut_subcommand("dc", |sub_cmd| {
            sub_cmd.mut_subcommand("stop", |sub_sub_cmd| {
                sub_sub_cmd
                    .mut_arg("wildfly-version", |arg| {
                        arg.value_parser(parse_version_enumeration)
                            .add(ArgValueCompleter::new(complete_running_versions(vec![
                                DomainController,
                            ])))
                    })
                    .mut_arg("name", |arg| {
                        arg.add(ArgValueCompleter::new(complete_running_names(vec![
                            DomainController,
                        ])))
                    })
            })
        })
        .mut_subcommand("hc", |sub_cmd| {
            sub_cmd.mut_subcommand("start", |sub_sub_cmd| {
                sub_sub_cmd
                    .mut_arg("wildfly-version", |arg| {
                        arg.value_parser(parse_version_enumeration)
                            .add(ArgValueCompleter::new(complete_versions))
                    })
                    .mut_arg("server", |arg| arg.value_parser(parse_servers))
                    .mut_arg("domain-controller", |arg| {
                        arg.add(ArgValueCompleter::new(complete_running_names(vec![
                            DomainController,
                        ])))
                    })
            })
        })
        .mut_subcommand("hc", |sub_cmd| {
            sub_cmd.mut_subcommand("stop", |sub_sub_cmd| {
                sub_sub_cmd
                    .mut_arg("wildfly-version", |arg| {
                        arg.value_parser(parse_version_enumeration)
                            .add(ArgValueCompleter::new(complete_running_versions(vec![
                                HostController,
                            ])))
                    })
                    .mut_arg("name", |arg| {
                        arg.add(ArgValueCompleter::new(complete_running_names(vec![
                            HostController,
                        ])))
                    })
            })
        })
        .mut_subcommand("topology", |sub_cmd| {
            sub_cmd
                .mut_subcommand("start", |sub_sub_cmd| {
                    sub_sub_cmd.mut_arg("setup", |arg| arg.value_parser(value_parser!(PathBuf)))
                })
                .mut_subcommand("stop", |sub_sub_cmd| {
                    sub_sub_cmd.mut_arg("setup", |arg| {
                        arg.add(ArgValueCompleter::new(complete_running_topologies()))
                    })
                })
        })
        .mut_subcommand("console", |sub_cmd| {
            sub_cmd
                .mut_arg("wildfly-version", |arg| {
                    arg.value_parser(parse_version_enumeration)
                        .add(ArgValueCompleter::new(complete_running_versions(vec![
                            Standalone,
                            DomainController,
                        ])))
                })
                .mut_arg("name", |arg| {
                    arg.add(ArgValueCompleter::new(complete_running_names(vec![
                        Standalone,
                        DomainController,
                    ])))
                })
        })
        .mut_subcommand("cli", |sub_cmd| {
            sub_cmd
                .mut_arg("wildfly-version", |arg| {
                    arg.value_parser(parse_version).add(ArgValueCompleter::new(
                        complete_running_versions(vec![Standalone, DomainController]),
                    ))
                })
                .mut_arg("name", |arg| {
                    arg.add(ArgValueCompleter::new(complete_running_names(vec![
                        Standalone,
                        DomainController,
                    ])))
                })
        })
}

//noinspection DuplicatedCode
#[tokio::main]
async fn main() -> Result<()> {
    clap_complete::CompleteEnv::with_factory(build_app_full).complete();

    let matches = build_app_full().get_matches();
    match matches.subcommand() {
        Some(("build", m)) => build(m).await,
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

        Some(("images", _)) => images(),
        Some(("ps", m)) => ps(m),
        Some(("console", m)) => console(m),
        Some(("cli", m)) => cli(m),
        Some(("completions", m)) => completions(m),

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
