use clap::ArgMatches;
use wildfly_container_versions::{WildFlyContainer, VERSIONS};

pub fn version_completion(matches: &ArgMatches) -> anyhow::Result<()> {
    if let Some(wildfly_version) = matches
        .get_one::<String>("wildfly-version")
        .map(|v| v.as_str())
    {
        let (prefix, token) = if let Some(last_comma_pos) = wildfly_version.rfind(',') {
            let (first, last) = wildfly_version.split_at(last_comma_pos);
            (first, &last[1..])
        } else {
            ("", wildfly_version)
        };
        if WildFlyContainer::version(token).is_ok() {
            println!("{},", prefix);
            println!("{}..", prefix);
            println!("{}x", prefix);
        } else {
            print_versions(Some(prefix), None, find_versions(token).as_slice());
        }
    } else {
        print_versions(None, None, all_versions().as_slice());
    }
    Ok(())
}

fn print_versions(prefix: Option<&str>, suffix: Option<&str>, versions: &[WildFlyContainer]) {
    for v in versions {
        if v.version.minor == 0 {
            println!(
                "{}{}{}",
                prefix.unwrap_or(""),
                v.version.major,
                suffix.unwrap_or("")
            );
        } else {
            println!(
                "{}{}.{}{}",
                prefix.unwrap_or(""),
                v.version.major,
                v.version.minor,
                suffix.unwrap_or("")
            );
        }
    }
}

fn find_versions(token: &str) -> Vec<WildFlyContainer> {
    if token.ends_with("..") {
        // "23..", "26.1..", "1..", "foo..", ".."
        if let Some(start) = right_substring(token, 2) {
            // "23..", "26.1..", "1..", "foo.."
            if let Ok(version) = WildFlyContainer::version(start.as_str()) {
                // "23..", "26.1.."
                versions_gt(version)
            } else {
                // "1..", "foo.."
                all_versions()
            }
        } else {
            // ".."
            all_versions().iter().skip(1).cloned().collect()
        }
    } else if token.ends_with('.') {
        // "23.", "26.1.", "30.1.2.", "1.", "foo.", "."
        if let Some(part) = right_substring(token, 1) {
            // "23.", "26.1.", "30.1.2.", "1.", "foo."
            if let Ok(version) = WildFlyContainer::version(part.as_str()) {
                // "23.", "26.1."
                if version.version.minor == 0 {
                    versions_with_major_like(version.version.major)
                } else {
                    vec![]
                }
            } else {
                // "30.1.2.", "1.", "foo."
                all_versions()
            }
        } else {
            // "."
            all_versions()
        }
    } else {
        all_versions()
    }
}

fn all_versions() -> Vec<WildFlyContainer> {
    VERSIONS.values().cloned().collect()
}

fn versions_gt(version: WildFlyContainer) -> Vec<WildFlyContainer> {
    VERSIONS
        .values()
        .filter(|&wfc| wfc.version > version.version)
        .cloned()
        .collect()
}

fn versions_with_major_like(major: u64) -> Vec<WildFlyContainer> {
    VERSIONS
        .values()
        .filter(|&wfc| wfc.version.major == major)
        .cloned()
        .collect()
}

fn right_substring(s: &str, len: usize) -> Option<String> {
    if len == 0 {
        Some(s.to_string())
    } else if len > s.len() {
        None
    } else {
        let chars = s.len() - len;
        Some(s.chars().take(chars).collect())
    }
}

#[cfg(test)]
mod completion_tests {
    use super::*;
    use wildfly_container_versions::WildFlyContainer;

    #[test]
    fn test_find_versions_like() {
        let versions = find_versions("2");
        assert_eq!(versions.len(), 11);
        assert!(
            versions
                .iter()
                .all(|v| v.version.major >= 20 && v.version.major < 30)
        );
    }

    #[test]
    fn test_find_versions_exact() {
        let versions = find_versions("26");
        assert_eq!(versions.len(), 2);
        assert!(
            versions
                .iter()
                .any(|v| v.version.major == 26 && v.version.minor == 0)
        );
        assert!(
            versions
                .iter()
                .any(|v| v.version.major == 26 && v.version.minor == 1)
        );
    }

    #[test]
    fn test_find_versions_major() {
        let versions = find_versions("26.");
        assert_eq!(versions.len(), 2);
        assert!(
            versions
                .iter()
                .any(|v| v.version.major == 26 && v.version.minor == 0)
        );
        assert!(
            versions
                .iter()
                .any(|v| v.version.major == 26 && v.version.minor == 1)
        );
    }

    #[test]
    fn test_find_versions_range() {
        let versions = find_versions("26..");
        assert_eq!(
            versions.len(),
            WildFlyContainer::range("26..").unwrap_or_default().len()
        );
        assert!(versions.iter().all(|v| v.version.major >= 26));
    }

    #[test]
    fn test_find_versions_multiplier() {
        let versions = find_versions("3x");
        assert_eq!(versions, all_versions());
    }

    #[test]
    fn test_find_versions_empty() {
        let versions = find_versions("");
        assert_eq!(versions, all_versions());
    }

    #[test]
    fn test_find_versions_invalid() {
        let versions = find_versions("invalid");
        assert_eq!(versions, all_versions());
    }

    #[test]
    fn test_right_substring() {
        assert_eq!(right_substring("hello world", 12), None);
        assert_eq!(right_substring("hello world", 11), Some("".to_string()));
        assert_eq!(right_substring("hello world", 10), Some("h".to_string()));
        assert_eq!(right_substring("hello world", 9), Some("he".to_string()));
        assert_eq!(right_substring("hello world", 8), Some("hel".to_string()));
        assert_eq!(right_substring("hello world", 7), Some("hell".to_string()));
        assert_eq!(right_substring("hello world", 6), Some("hello".to_string()));
        assert_eq!(
            right_substring("hello world", 5),
            Some("hello ".to_string())
        );
        assert_eq!(
            right_substring("hello world", 4),
            Some("hello w".to_string())
        );
        assert_eq!(
            right_substring("hello world", 3),
            Some("hello wo".to_string())
        );
        assert_eq!(
            right_substring("hello world", 2),
            Some("hello wor".to_string())
        );
        assert_eq!(
            right_substring("hello world", 1),
            Some("hello worl".to_string())
        );
        assert_eq!(
            right_substring("hello world", 0),
            Some("hello world".to_string())
        );
    }
}
