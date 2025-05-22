use clap::ArgMatches;
use semver::Version;
use wildfly_container_versions::VERSIONS;

pub fn version_completion(matches: &ArgMatches) -> anyhow::Result<()> {
    if let Some(parameter) = matches
        .get_one::<String>("wildfly-version")
        .map(|v| v.as_str())
    {
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
            print_suggestions(prefix, token, &all_versions());
        } else if token == ".." {
            print_suggestions(
                prefix,
                token,
                &all_versions()
                    .iter()
                    .skip(1)
                    .cloned()
                    .collect::<Vec<String>>(),
            );
        } else if token.starts_with("..") {
            let after_dots = token.chars().skip(2).collect::<String>();
            print_suggestions(
                prefix,
                token,
                &all_versions()
                    .iter()
                    .filter(|v| v.starts_with(&after_dots))
                    .map(|v| v.trim_start_matches(&after_dots).to_string())
                    .filter(|v| !v.is_empty())
                    .collect::<Vec<String>>(),
            );
        } else if token.ends_with("..") {
            let before_dots = token.chars().take(token.len() - 2).collect::<String>();
            print_suggestions(
                prefix,
                token,
                &all_versions()
                    .iter()
                    .filter(|v| v.gt(&&before_dots))
                    .cloned()
                    .collect::<Vec<String>>(),
            );
        } else if token.contains("..") {
            let after_dots = token.split("..").nth(1).unwrap_or("");
            print_suggestions(
                prefix,
                token,
                &all_versions()
                    .iter()
                    .filter(|v| v.starts_with(after_dots))
                    .map(|v| v.trim_start_matches(after_dots).to_string())
                    .filter(|v| !v.is_empty())
                    .collect::<Vec<String>>(),
            );
        } else {
            print_suggestions(prefix, "", &all_versions());
        }
    } else {
        print_suggestions("", "", &all_versions());
    }
    Ok(())
}

fn print_suggestions(prefix_0: &str, prefix_1: &str, suggestions: &[String]) {
    for s in suggestions {
        println!("{}{}{}", prefix_0, prefix_1, s);
    }
}

fn all_versions() -> Vec<String> {
    VERSIONS
        .values()
        .map(|wfc| simple_version(&wfc.version))
        .collect()
}

fn simple_version(version: &Version) -> String {
    if version.minor == 0 {
        format!("{}", version.major)
    } else {
        format!("{}.{}", version.major, version.minor)
    }
}
