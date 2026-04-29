mod app;
mod args;
mod command;
mod completion;
mod constants;
mod container;
mod label;
mod progress;
mod resources;
mod wildfly;

use crate::command::build::build;
use crate::command::cli::cli;
use crate::command::completions::completions;
use crate::command::console::console;
use crate::command::dc::{dc_start, dc_stop};
use crate::command::hc::{hc_start, hc_stop};
use crate::command::images::images;
use crate::command::ps::ps;
use crate::command::push::push;
use crate::command::standalone::{standalone_start, standalone_stop};
use crate::command::topology::{topology_start, topology_stop};
use crate::command::update::update;
use crate::command::versions::versions;
use crate::completion::{
    complete_running_names, complete_running_topologies, complete_running_versions,
    complete_versions,
};
use crate::wildfly::Server;
use crate::wildfly::ServerType::{DomainController, HostController, Standalone};
use anyhow::Result;
use app::build_app;
use clap::value_parser;
use clap_complete::engine::ArgValueCompleter;
use std::path::PathBuf;
use wildfly_meta::{FeaturePackRegistry, WildFlyImage, WildFlyImageRegistry, parse_image, parse_list, ParseOptions, MetaItem};

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

    let registry = WildFlyImageRegistry::load_default()?;
    let matches = build_app_full().get_matches();
    match matches.subcommand() {
        Some(("build", m)) => build(m).await,
        Some(("push", m)) => push(m),

        Some(("start", m)) => standalone_start(m, &registry),
        Some(("stop", m)) => standalone_stop(m, &registry),

        Some(("dc", sub_matches)) => match sub_matches.subcommand() {
            Some(("start", m)) => dc_start(m, &registry),
            Some(("stop", m)) => dc_stop(m, &registry),
            _ => unreachable!("Unknown subcommand"),
        },

        Some(("hc", sub_matches)) => match sub_matches.subcommand() {
            Some(("start", m)) => hc_start(m, &registry),
            Some(("stop", m)) => hc_stop(m, &registry),
            _ => unreachable!("Unknown subcommand"),
        },

        Some(("topology", sub_matches)) => match sub_matches.subcommand() {
            Some(("start", m)) => topology_start(m, &registry),
            Some(("stop", m)) => topology_stop(m, &registry),
            _ => unreachable!("Unknown subcommand"),
        },

        Some(("images", _)) => images(&registry),
        Some(("ps", m)) => ps(m, &registry),
        Some(("console", m)) => console(m, &registry),
        Some(("cli", m)) => cli(m, &registry),
        Some(("completions", m)) => completions(m),
        Some(("update", _)) => update(),
        Some(("versions", _)) => versions(&registry),

        _ => unreachable!("Unknown subcommand"),
    }?;
    Ok(())
}

// ------------------------------------------------------ validation

fn parse_version_enumeration(range: &str) -> Result<Vec<WildFlyImage>, String> {
    let registry = WildFlyImageRegistry::load_default().map_err(|e| e.to_string())?;
    let packs = FeaturePackRegistry::load_default()
        .unwrap_or_else(|_| FeaturePackRegistry::from_toml("config_version = 1\nfeature_packs = []").unwrap());
    let items = parse_list(range, &registry, &packs, &ParseOptions::all()).map_err(|e| e.to_string())?;
    let images: Vec<WildFlyImage> = items
        .into_iter()
        .filter_map(|item| match item {
            MetaItem::Image(img) => Some(img),
            _ => None,
        })
        .collect();
    if images.is_empty() {
        Err(format!("No valid WildFly versions found in '{}'", range))
    } else {
        Ok(images)
    }
}

fn parse_version(version: &str) -> Result<WildFlyImage, String> {
    let registry = WildFlyImageRegistry::load_default().map_err(|e| e.to_string())?;
    parse_image(version, &registry).map_err(|e| e.to_string())
}

fn parse_servers(server: &str) -> Result<Vec<Server>, String> {
    Server::parse_servers(server).map_err(|err| err.to_string())
}
