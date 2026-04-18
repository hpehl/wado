use std::collections::BTreeSet;
use std::ffi::OsStr;

use clap_complete::engine::CompletionCandidate;
use futures::executor::block_on;
use semver::Version;
use wildfly_container_versions::{VERSIONS, WildFlyContainer};

use crate::container::container_ps;
use crate::wildfly::ServerType;

pub fn complete_versions(current: &OsStr) -> Vec<CompletionCandidate> {
    let input = current.to_str().unwrap_or("");
    let parameter = if input.is_empty() { None } else { Some(input) };
    let (prefix_0, prefix_1, suggestions) = find_suggestions(parameter);
    suggestions
        .iter()
        .map(|s| CompletionCandidate::new(format!("{}{}{}", prefix_0, prefix_1, s)))
        .collect()
}

pub fn complete_running_names(
    server_types: Vec<ServerType>,
) -> impl Fn(&OsStr) -> Vec<CompletionCandidate> {
    move |_current: &OsStr| {
        let instances = block_on(container_ps(server_types.clone(), None, None, false));
        match instances {
            Ok(instances) => instances
                .iter()
                .map(|i| CompletionCandidate::new(i.name.clone()))
                .collect(),
            Err(_) => vec![],
        }
    }
}

pub fn complete_running_versions(
    server_types: Vec<ServerType>,
) -> impl Fn(&OsStr) -> Vec<CompletionCandidate> {
    move |current: &OsStr| {
        let input = current.to_str().unwrap_or("");
        let (prefix, _token) =
            parse_prefix_token(if input.is_empty() { None } else { Some(input) });
        let instances = block_on(container_ps(server_types.clone(), None, None, false));
        match instances {
            Ok(instances) => {
                let versions: BTreeSet<String> = instances
                    .iter()
                    .map(|i| i.admin_container.wildfly_container.display_version())
                    .collect();
                versions
                    .iter()
                    .map(|v| CompletionCandidate::new(format!("{}{}", prefix, v)))
                    .collect()
            }
            Err(_) => vec![],
        }
    }
}

fn find_suggestions(parameter: Option<&str>) -> (String, String, Vec<String>) {
    let (prefix, token) = parse_prefix_token(parameter);

    let (out_token, suggestions): (&str, Vec<String>) = if token.ends_with('x') {
        (token, all_simple_versions())
    } else if token == ".." {
        let versions = all_simple_versions().into_iter().skip(1).collect();
        (token, versions)
    } else if let Some(after) = token.strip_prefix("..") {
        (token, suggest_after_dots(after, &Version::new(0, 0, 0)))
    } else if let Some(before) = token.strip_suffix("..") {
        let versions = parse_version(before)
            .map(|v| versions_after(&v))
            .unwrap_or_default();
        (token, versions)
    } else if token.contains("..") {
        let (before, after) = token.split_once("..").unwrap_or(("", ""));
        let versions = parse_version(before)
            .map(|v| suggest_after_dots(after, &v))
            .unwrap_or_default();
        (token, versions)
    } else {
        ("", all_simple_versions())
    };

    (prefix.to_string(), out_token.to_string(), suggestions)
}

fn parse_prefix_token(parameter: Option<&str>) -> (&str, &str) {
    match parameter {
        Some(param) => match param.rfind(',') {
            Some(pos) if pos < param.len() - 1 => param.split_at(pos + 1),
            Some(_) => (param, ""),
            None => ("", param),
        },
        None => ("", ""),
    }
}

fn parse_version(input: &str) -> Option<Version> {
    WildFlyContainer::version(input).ok().map(|wfc| wfc.version)
}

fn versions_after(start: &Version) -> Vec<String> {
    all_versions()
        .iter()
        .filter(|v| {
            if v.major == start.major {
                v.minor > start.minor
            } else {
                v.major > start.major
            }
        })
        .map(simple_version)
        .collect()
}

fn suggest_after_dots(after_dots: &str, start_after: &Version) -> Vec<String> {
    if WildFlyContainer::version(after_dots).is_ok() {
        return vec![];
    }

    let major_number = after_dots
        .strip_suffix('.')
        .unwrap_or(after_dots)
        .parse::<u64>()
        .ok();

    if let Some(number) = major_number {
        let versions = all_versions();
        let filtered: Vec<String> = versions
            .iter()
            .skip_while(|v| v <= &start_after)
            .filter(|v| match number {
                1..=9 if !after_dots.ends_with('.') => {
                    v.major >= (number * 10) && v.major < ((number + 1) * 10)
                }
                _ => v.major == number && v.minor > 0,
            })
            .map(simple_version)
            .map(|v| v.strip_prefix(after_dots).unwrap_or(&v).to_string())
            .collect();
        filtered
    } else {
        vec![]
    }
}

fn all_versions() -> Vec<Version> {
    VERSIONS.values().map(|wfc| wfc.version.clone()).collect()
}

fn all_simple_versions() -> Vec<String> {
    let mut versions: Vec<String> = all_versions().iter().map(simple_version).collect();
    if WildFlyContainer::version("dev").is_ok() {
        versions.push("dev".to_string());
    }
    versions
}

fn simple_version(version: &Version) -> String {
    if version.minor == 0 {
        format!("{}", version.major)
    } else {
        format!("{}.{}", version.major, version.minor)
    }
}

// ------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_suggestions_empty() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(None);
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "");
        assert_eq!(suggestions, all_simple_versions());
    }

    #[test]
    fn test_find_suggestions_invalid() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("--foo"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "");
        assert_eq!(suggestions, all_simple_versions());
    }

    #[test]
    fn test_find_suggestions_with_x() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("3x"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "3x");
        assert_eq!(suggestions, all_simple_versions());
    }

    #[test]
    fn test_find_suggestions_with_dots() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some(".."));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "..");
        let expected: Vec<String> = all_simple_versions().into_iter().skip(1).collect();
        assert_eq!(suggestions, expected);
    }

    #[test]
    fn test_find_suggestions_with_dots_2() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("..2"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "..2");
        assert_eq!(
            suggestions,
            ["0", "1", "2", "3", "4", "5", "6", "6.1", "7", "8", "9"]
        );
    }

    #[test]
    fn test_find_suggestions_with_dots_20() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("..20"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "..20");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_find_suggestions_with_dots_26_dot() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("..26."));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "..26.");
        assert_eq!(suggestions, ["1"]);
    }

    #[test]
    fn test_find_suggestions_with_dots_1000() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("..1000"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "..1000");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_find_suggestions_with_dots_1000_dot() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("..1000."));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "..1000.");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_find_suggestions_with_dots_foo() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("..foo"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "..foo");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_find_suggestions_with_10_dots() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("10.."));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "10..");
        assert_eq!(
            suggestions,
            all_versions()
                .iter()
                .skip(1)
                .map(simple_version)
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_find_suggestions_with_101_dots() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("10.1.."));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "10.1..");
        assert_eq!(
            suggestions,
            all_versions()
                .iter()
                .skip(2)
                .map(simple_version)
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_find_suggestions_with_1000_dots() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("1000.."));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "1000..");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_find_suggestions_with_foo_dots() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("foo.."));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "foo..");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_find_suggestions_with_26_dots_1() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("26..1"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "26..1");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_find_suggestions_with_26_dots_2() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("26..2"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "26..2");
        assert_eq!(suggestions, ["6.1", "7", "8", "9"]);
    }

    #[test]
    fn test_find_suggestions_with_26_dots_9() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("26..9"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "26..9");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_find_suggestions_with_261_dots_2() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("26.1..2"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "26.1..2");
        assert_eq!(suggestions, ["7", "8", "9"]);
    }

    #[test]
    fn test_find_suggestions_with_1_dots_26() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("1..26"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "1..26");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_find_suggestions_with_1000_dots_26() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("1000..26"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "1000..26");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_find_suggestions_with_foo_dots_26() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("foo..26"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "foo..26");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_find_suggestions_with_trailing_comma() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("26,"));
        assert_eq!(prefix_0, "26,");
        assert_eq!(prefix_1, "");
        assert_eq!(suggestions, all_simple_versions());
    }

    #[test]
    fn test_find_suggestions_with_comma_and_token() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("26,2"));
        assert_eq!(prefix_0, "26,");
        assert_eq!(prefix_1, "");
        assert_eq!(suggestions, all_simple_versions());
    }

    #[test]
    fn test_find_suggestions_with_comma_and_dots() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("26,..2"));
        assert_eq!(prefix_0, "26,");
        assert_eq!(prefix_1, "..2");
        assert_eq!(
            suggestions,
            ["0", "1", "2", "3", "4", "5", "6", "6.1", "7", "8", "9"]
        );
    }

    #[test]
    fn test_find_suggestions_with_multiple_commas() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("10,26,..2"));
        assert_eq!(prefix_0, "10,26,");
        assert_eq!(prefix_1, "..2");
        assert_eq!(
            suggestions,
            ["0", "1", "2", "3", "4", "5", "6", "6.1", "7", "8", "9"]
        );
    }

    #[test]
    fn test_find_suggestions_with_comma_and_x() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("26,3x"));
        assert_eq!(prefix_0, "26,");
        assert_eq!(prefix_1, "3x");
        assert_eq!(suggestions, all_simple_versions());
    }
}
