use std::ffi::OsStr;
use std::sync::OnceLock;

use clap_complete::engine::CompletionCandidate;
use wildfly_meta::{DslOptions, WildFlyImageRegistry, suggest_wildfly_images};

static REGISTRY: OnceLock<Option<WildFlyImageRegistry>> = OnceLock::new();

fn registry() -> Option<&'static WildFlyImageRegistry> {
    REGISTRY
        .get_or_init(|| WildFlyImageRegistry::load_default("Run 'wado update' to fix this.").ok())
        .as_ref()
}

pub fn complete_versions(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(registry) = registry() else {
        return vec![];
    };
    let input = current.to_str().unwrap_or("");
    suggest_wildfly_images(input, registry, &DslOptions::all())
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

pub fn parse_prefix_token(parameter: Option<&str>) -> (&str, &str) {
    match parameter {
        Some(param) => match param.rfind(',') {
            Some(pos) if pos < param.len() - 1 => param.split_at(pos + 1),
            Some(_) => (param, ""),
            None => ("", param),
        },
        None => ("", ""),
    }
}
