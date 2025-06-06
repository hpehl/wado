use clap::ArgMatches;
use semver::Version;
use wildfly_container_versions::{WildFlyContainer, VERSIONS};

pub fn version_completion(matches: &ArgMatches) -> anyhow::Result<()> {
    match matches.try_get_one::<String>("wildfly-version") {
        Ok(result) => {
            let (prefix_0, prefix_1, suggestions) = find_suggestions(result.map(|v| v.as_str()));
            print_suggestions(prefix_0, prefix_1, &suggestions);
        }
        Err(_) => {
            // ignore any error!
        }
    }
    Ok(())
}

fn find_suggestions(parameter: Option<&str>) -> (String, String, Vec<String>) {
    if let Some(parameter) = parameter {
        let (prefix, token) = if let Some(last_comma_pos) = parameter.rfind(',') {
            if last_comma_pos < parameter.len() - 1 {
                let (first, last) = parameter.split_at(last_comma_pos + 1);
                (first, last)
            } else {
                (parameter, "")
            }
        } else {
            ("", parameter)
        };

        if token.ends_with('x') {
            (
                prefix.to_string(),
                token.to_string(),
                all_versions()
                    .iter()
                    .map(simple_version)
                    .collect::<Vec<String>>(),
            )
        } else if token == ".." {
            (
                prefix.to_string(),
                token.to_string(),
                all_versions()
                    .iter()
                    .skip(1)
                    .map(simple_version)
                    .collect::<Vec<String>>(),
            )
        } else if token.starts_with("..") {
            let after_dots = token.chars().skip(2).collect::<String>();
            find_after_dots(prefix, token, after_dots.as_str(), &Version::new(0, 0, 0))
        } else if token.ends_with("..") {
            let before_dots = token.chars().take(token.len() - 2).collect::<String>();
            if let Ok(version) =
                WildFlyContainer::version(before_dots.as_str()).map(|wfc| wfc.version)
            {
                (
                    prefix.to_string(),
                    token.to_string(),
                    all_versions()
                        .iter()
                        .filter(|v| {
                            if v.major == version.major {
                                v.minor > version.minor
                            } else {
                                v.major > version.major
                            }
                        })
                        .map(simple_version)
                        .collect::<Vec<String>>(),
                )
            } else {
                (prefix.to_string(), token.to_string(), vec![])
            }
        } else if token.contains("..") {
            let before_dots = token.split("..").next().unwrap_or("");
            if let Ok(version) = WildFlyContainer::version(before_dots).map(|wfc| wfc.version) {
                let after_dots = token.split("..").nth(1).unwrap_or("");
                find_after_dots(prefix, token, after_dots, &version)
            } else {
                (prefix.to_string(), token.to_string(), vec![])
            }
        } else {
            (
                prefix.to_string(),
                "".to_string(),
                all_versions()
                    .iter()
                    .map(simple_version)
                    .collect::<Vec<String>>(),
            )
        }
    } else {
        (
            "".to_string(),
            "".to_string(),
            all_versions()
                .iter()
                .map(simple_version)
                .collect::<Vec<String>>(),
        )
    }
}

fn find_after_dots(
    prefix: &str,
    token: &str,
    after_dots: &str,
    start_after: &Version,
) -> (String, String, Vec<String>) {
    if WildFlyContainer::version(after_dots).is_ok() {
        (prefix.to_string(), token.to_string(), vec![])
    } else if let Ok(number) = after_dots.parse::<u64>() {
        match number {
            1..=9 => (
                prefix.to_string(),
                token.to_string(),
                all_versions()
                    .iter()
                    .skip_while(|v| v <= &start_after)
                    .filter(|v| v.major >= (number * 10) && v.major < ((number + 1) * 10))
                    .map(simple_version)
                    .map(|v| v.strip_prefix(after_dots).unwrap_or(&v).to_string())
                    .collect::<Vec<String>>(),
            ),
            10.. => (
                prefix.to_string(),
                token.to_string(),
                all_versions()
                    .iter()
                    .skip_while(|v| v <= &start_after)
                    .filter(|v| v.major == number && v.minor > 0)
                    .map(simple_version)
                    .map(|v| v.strip_prefix(after_dots).unwrap_or(&v).to_string())
                    .collect::<Vec<String>>(),
            ),
            _ => (prefix.to_string(), token.to_string(), vec![]),
        }
    } else if after_dots.ends_with('.') {
        if let Ok(number) = &after_dots[0..after_dots.len() - 1].parse::<u64>() {
            (
                prefix.to_string(),
                token.to_string(),
                all_versions()
                    .iter()
                    .skip_while(|v| v <= &start_after)
                    .filter(|v| v.major == *number && v.minor > 0)
                    .map(simple_version)
                    .map(|v| v.strip_prefix(after_dots).unwrap_or(&v).to_string())
                    .collect::<Vec<String>>(),
            )
        } else {
            (prefix.to_string(), token.to_string(), vec![])
        }
    } else {
        (prefix.to_string(), token.to_string(), vec![])
    }
}

fn print_suggestions(prefix_0: String, prefix_1: String, suggestions: &[String]) {
    for s in suggestions {
        println!("{}{}{}", prefix_0, prefix_1, s);
    }
}

fn all_versions() -> Vec<Version> {
    VERSIONS.values().map(|wfc| wfc.version.clone()).collect()
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
        assert_eq!(
            suggestions,
            all_versions()
                .iter()
                .map(simple_version)
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_find_suggestions_invalid() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("--foo"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "");
        assert_eq!(
            suggestions,
            all_versions()
                .iter()
                .map(simple_version)
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_find_suggestions_with_x() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some("3x"));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "3x");
        assert_eq!(
            suggestions,
            all_versions()
                .iter()
                .map(simple_version)
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn test_find_suggestions_with_dots() {
        let (prefix_0, prefix_1, suggestions) = find_suggestions(Some(".."));
        assert_eq!(prefix_0, "");
        assert_eq!(prefix_1, "..");
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

    // TODO Test commas
}
