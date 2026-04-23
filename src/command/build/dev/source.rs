use super::task::DevTask;
use crate::container::container_command;
use futures::future::Either;
use indicatif::MultiProgress;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::LazyLock;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

static HAL_JAR_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"path="hal-console-[^"]*-resources\.jar""#).expect("invalid HAL jar regex")
});

const MAVEN_CACHE_VOLUME: &str = "wado-maven-cache";
const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024;

const WILDFLY_BUILD_IMAGE: &str = "maven:3.9-eclipse-temurin-21";
const WILDFLY_REPO: &str = "https://github.com/wildfly/wildfly.git";
const WILDFLY_MAVEN_ARGS: &[&str] = &["-pl", "dist", "-am"];

const HAL_BUILD_IMAGE: &str = "maven:3.9-eclipse-temurin-11";
const HAL_REPO: &str = "https://github.com/hal/console.git";
const HAL_MAVEN_ARGS: &[&str] = &["-P", "prod,theme-wildfly"];

// ------------------------------------------------------ maven command

fn build_maven_command(
    repo_url: &str,
    branch: &str,
    maven_args: &[&str],
    build_image: &str,
    volume_name: &str,
) -> anyhow::Result<Command> {
    let maven_extra = maven_args.join(" ");
    let script = "git clone --depth 1 -b \"$WADO_BRANCH\" \"$WADO_REPO\" /build && \
         cd /build && \
         mvn -B install -DskipTests -Denforcer.skip=true $WADO_MAVEN_ARGS";

    let mut cmd = container_command()?;
    cmd.arg("run")
        .arg("--rm")
        .arg("-e")
        .arg(format!("WADO_BRANCH={}", branch))
        .arg("-e")
        .arg(format!("WADO_REPO={}", repo_url))
        .arg("-e")
        .arg(format!("WADO_MAVEN_ARGS={}", maven_extra))
        .arg("-v")
        .arg(format!("{}:/build", volume_name))
        .arg("-v")
        .arg(format!("{}:/root/.m2", MAVEN_CACHE_VOLUME))
        .arg("-w")
        .arg("/build")
        .arg(build_image)
        .arg("sh")
        .arg("-c")
        .arg(script);

    Ok(cmd)
}

// ------------------------------------------------------ clone and build (progress)

pub(super) async fn clone_and_build_repos(
    wildfly_branch: &str,
    hal_branch: &str,
    wf_volume: &str,
    hal_volume: &str,
) -> anyhow::Result<()> {
    let multi = MultiProgress::new();
    let wf_task = DevTask::new(&multi, "WildFly");
    let hal_task = DevTask::new(&multi, "HAL console");

    let wf_fut = Box::pin(clone_and_build_repo(
        wf_task,
        WILDFLY_REPO,
        wildfly_branch,
        WILDFLY_MAVEN_ARGS,
        WILDFLY_BUILD_IMAGE,
        wf_volume,
    ));
    let hal_fut = Box::pin(clone_and_build_repo(
        hal_task,
        HAL_REPO,
        hal_branch,
        HAL_MAVEN_ARGS,
        HAL_BUILD_IMAGE,
        hal_volume,
    ));

    match futures::future::select(wf_fut, hal_fut).await {
        Either::Left(((wf_task, wf_result), hal_remaining)) => {
            if wf_result.is_err() {
                wf_task.print_errors();
                drop(hal_remaining);
                anyhow::bail!("Build failed")
            }
            let (hal_task, hal_result) = hal_remaining.await;
            if hal_result.is_err() {
                hal_task.print_errors();
                anyhow::bail!("Build failed")
            }
        }
        Either::Right(((hal_task, hal_result), wf_remaining)) => {
            if hal_result.is_err() {
                hal_task.print_errors();
                drop(wf_remaining);
                anyhow::bail!("Build failed")
            }
            let (wf_task, wf_result) = wf_remaining.await;
            if wf_result.is_err() {
                wf_task.print_errors();
                anyhow::bail!("Build failed")
            }
        }
    }

    Ok(())
}

async fn clone_and_build_repo(
    mut task: DevTask,
    repo_url: &str,
    branch: &str,
    maven_args: &[&str],
    build_image: &str,
    volume_name: &str,
) -> (DevTask, anyhow::Result<()>) {
    let result = clone_and_build_repo_inner(
        &mut task,
        repo_url,
        branch,
        maven_args,
        build_image,
        volume_name,
    )
    .await;
    if result.is_err() && !task.finished {
        task.finish_error("failed");
    }
    (task, result)
}

async fn clone_and_build_repo_inner(
    task: &mut DevTask,
    repo_url: &str,
    branch: &str,
    maven_args: &[&str],
    build_image: &str,
    volume_name: &str,
) -> anyhow::Result<()> {
    task.set_progress(&format!("cloning ({})...", branch));

    let log_path = std::env::temp_dir().join(format!(
        "wado-{}.log",
        task.name.to_lowercase().replace(' ', "-")
    ));
    let mut log_file = BufWriter::new(File::create(&log_path)?);
    task.log_path = Some(log_path.clone());

    let mut child = build_maven_command(repo_url, branch, maven_args, build_image, volume_name)?
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let last_counter = stream_build_output(&mut child, task, &mut log_file, branch).await?;

    drop(log_file);
    let status = child.wait().await?;
    if status.success() {
        let detail = if last_counter.is_empty() {
            Some(format!("({})", branch))
        } else {
            Some(format!("{} ({})", last_counter, branch))
        };
        task.finish_success(detail.as_deref());
        if let Err(e) = fs::remove_file(&log_path) {
            eprintln!(
                "Warning: failed to remove log file {}: {}",
                log_path.display(),
                e
            );
        }
        task.log_path = None;
        Ok(())
    } else {
        task.finish_error("build failed");
        anyhow::bail!("Clone/build failed for {}", repo_url)
    }
}

async fn stream_build_output(
    child: &mut tokio::process::Child,
    task: &mut DevTask,
    log_file: &mut BufWriter<File>,
    branch: &str,
) -> anyhow::Result<String> {
    let stdout = child.stdout.take().expect("No stdout handle");
    let stderr = child.stderr.take().expect("No stderr handle");
    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut stderr_lines = BufReader::new(stderr).lines();
    let mut stdout_done = false;
    let mut stderr_done = false;
    let mut cloning = true;
    let mut last_counter = String::new();
    let mut bytes_written: u64 = 0;

    loop {
        tokio::select! {
            result = stdout_lines.next_line(), if !stdout_done => {
                match result? {
                    Some(line) => {
                        write_log_line(log_file, &line, &mut bytes_written);
                        task.append_line(&line);
                        if cloning && line.contains("[INFO] Building ") {
                            cloning = false;
                        }
                        if !cloning
                            && let Some(progress) = parse_maven_module(&line)
                        {
                            if let Some((counter, _)) = progress.split_once(' ') {
                                last_counter = counter.to_string();
                            }
                            task.set_progress(&progress);
                        }
                    }
                    None => stdout_done = true,
                }
            }
            result = stderr_lines.next_line(), if !stderr_done => {
                match result? {
                    Some(line) => {
                        write_log_line(log_file, &line, &mut bytes_written);
                        task.append_line(&line);
                        if cloning && line.contains("Receiving objects:") {
                            task.set_progress(&format!("cloning ({})...", branch));
                        }
                    }
                    None => stderr_done = true,
                }
            }
        }
        if stdout_done && stderr_done {
            break;
        }
    }

    Ok(last_counter)
}

fn write_log_line(log_file: &mut BufWriter<File>, line: &str, bytes_written: &mut u64) {
    if *bytes_written < MAX_LOG_BYTES
        && let Ok(()) = writeln!(log_file, "{}", line)
    {
        *bytes_written += line.len() as u64 + 1;
    }
}

// ------------------------------------------------------ clone and build (verbose)

pub(super) async fn clone_and_build_repos_verbose(
    wildfly_branch: &str,
    hal_branch: &str,
    wf_volume: &str,
    hal_volume: &str,
) -> anyhow::Result<()> {
    clone_and_build_repo_verbose(
        "WildFly",
        WILDFLY_REPO,
        wildfly_branch,
        WILDFLY_MAVEN_ARGS,
        WILDFLY_BUILD_IMAGE,
        wf_volume,
    )
    .await?;
    clone_and_build_repo_verbose(
        "HAL console",
        HAL_REPO,
        hal_branch,
        HAL_MAVEN_ARGS,
        HAL_BUILD_IMAGE,
        hal_volume,
    )
    .await
}

async fn clone_and_build_repo_verbose(
    name: &str,
    repo_url: &str,
    branch: &str,
    maven_args: &[&str],
    build_image: &str,
    volume_name: &str,
) -> anyhow::Result<()> {
    println!("\n--- {} (branch: {}) ---", name, branch);

    let status = build_maven_command(repo_url, branch, maven_args, build_image, volume_name)?
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;

    if status.success() {
        println!("--- {} done ---\n", name);
        Ok(())
    } else {
        anyhow::bail!("Clone/build failed for {}", name)
    }
}

// ------------------------------------------------------ extract artifacts

async fn extract_from_volume(
    volume_name: &str,
    build_image: &str,
    find_command: &str,
    container_suffix: &str,
    artifact_dir: &Path,
) -> anyhow::Result<PathBuf> {
    // Find the artifact path inside the volume
    let mut cmd = container_command()?;
    cmd.arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(format!("{}:/build:ro", volume_name))
        .arg(build_image)
        .arg("sh")
        .arg("-c")
        .arg(find_command);

    let output = cmd.output().await?;
    if !output.status.success() {
        anyhow::bail!("Failed to find artifact in volume {}", volume_name);
    }

    let artifact_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if artifact_path.is_empty() {
        anyhow::bail!("Could not find artifact in volume {}", volume_name);
    }

    let artifact_name = Path::new(&artifact_path)
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid artifact path"))?
        .to_string_lossy()
        .to_string();

    // Create a temporary container to copy the artifact out of the volume
    let container_name = format!("wado-extract-{}-{}", container_suffix, std::process::id());
    let mut create_cmd = container_command()?;
    create_cmd
        .arg("create")
        .arg("--name")
        .arg(&container_name)
        .arg("-v")
        .arg(format!("{}:/build:ro", volume_name))
        .arg(build_image)
        .arg("true");
    let status = create_cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;
    if !status.success() {
        anyhow::bail!("Failed to create extraction container");
    }

    // Copy artifact to host
    let host_artifact = artifact_dir.join(&artifact_name);
    let mut cp_cmd = container_command()?;
    cp_cmd
        .arg("cp")
        .arg(format!(
            "{}:{}",
            container_name,
            artifact_path.trim_end_matches('/')
        ))
        .arg(&host_artifact);
    let cp_status = cp_cmd
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .await?;

    // Clean up extraction container
    let mut rm_cmd = container_command()?;
    if let Err(e) = rm_cmd
        .arg("rm")
        .arg("-f")
        .arg(&container_name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
    {
        eprintln!(
            "Warning: failed to remove container {}: {}",
            container_name, e
        );
    }

    if !cp_status.success() {
        anyhow::bail!("Failed to extract artifact from volume {}", volume_name);
    }

    Ok(host_artifact)
}

pub(super) async fn extract_wildfly_dist(
    volume_name: &str,
    artifact_dir: &Path,
) -> anyhow::Result<PathBuf> {
    extract_from_volume(
        volume_name,
        WILDFLY_BUILD_IMAGE,
        "ls -d /build/dist/target/wildfly-*/ 2>/dev/null | head -1",
        "wf",
        artifact_dir,
    )
    .await
}

pub(super) async fn extract_hal_jar(
    volume_name: &str,
    artifact_dir: &Path,
) -> anyhow::Result<PathBuf> {
    extract_from_volume(
        volume_name,
        HAL_BUILD_IMAGE,
        "ls /build/app/target/hal-console-*-resources.jar 2>/dev/null | head -1",
        "hal",
        artifact_dir,
    )
    .await
}

// ------------------------------------------------------ integrate

pub(super) fn integrate_hal(wildfly_dist: &Path, hal_jar: &Path) -> anyhow::Result<()> {
    let console_module_dir = wildfly_dist
        .join("modules")
        .join("system")
        .join("layers")
        .join("base")
        .join("org")
        .join("jboss")
        .join("as")
        .join("console")
        .join("main");

    if !console_module_dir.exists() {
        anyhow::bail!(
            "Console module directory not found: {}",
            console_module_dir.display()
        );
    }

    // Delete old HAL jar(s)
    for entry in fs::read_dir(&console_module_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("hal-console-") && name_str.ends_with("-resources.jar") {
            fs::remove_file(entry.path())?;
        }
    }

    // Copy new HAL jar
    let hal_jar_name = hal_jar
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid HAL jar path"))?;
    let dest = console_module_dir.join(hal_jar_name);
    fs::copy(hal_jar, &dest)?;

    // Update module.xml resource-root reference
    let module_xml_path = console_module_dir.join("module.xml");
    let content = fs::read_to_string(&module_xml_path)?;
    let hal_jar_name_str = hal_jar_name.to_string_lossy();
    let updated = HAL_JAR_REGEX
        .replace(&content, format!(r#"path="{}""#, hal_jar_name_str).as_str())
        .to_string();
    fs::write(&module_xml_path, updated)?;
    Ok(())
}

// ------------------------------------------------------ maven parsing

pub(super) fn parse_maven_module(line: &str) -> Option<String> {
    let marker = "[INFO] Building ";
    let start = line.find(marker)? + marker.len();
    let rest = &line[start..];

    let bracket_start = rest.rfind('[')?;
    let bracket_end = rest.rfind(']')?;
    if bracket_end <= bracket_start {
        return None;
    }

    let counter = &rest[bracket_start..=bracket_end];
    let inner = &rest[bracket_start + 1..bracket_end];
    let parts: Vec<&str> = inner.split('/').collect();
    if parts.len() != 2 || parts.iter().any(|p| p.trim().parse::<u32>().is_err()) {
        return None;
    }

    let name_with_version = rest[..bracket_start].trim();
    let module_name = match name_with_version.rsplit_once(' ') {
        Some((name, _version)) => name.trim(),
        None => name_with_version,
    };

    Some(format!("{} {}", counter, module_name))
}

// ------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_maven_line() {
        let line =
            "[INFO] Building WildFly: Naming Subsystem 40.0.0.Beta1-SNAPSHOT         [2/161]";
        assert_eq!(
            parse_maven_module(line),
            Some("[2/161] WildFly: Naming Subsystem".to_string())
        );
    }

    #[test]
    fn parse_hal_maven_line() {
        let line = "[INFO] Building HAL Console Parent 3.7.20-SNAPSHOT                      [1/12]";
        assert_eq!(
            parse_maven_module(line),
            Some("[1/12] HAL Console Parent".to_string())
        );
    }

    #[test]
    fn parse_single_word_module_name() {
        let line = "[INFO] Building foo [1/5]";
        assert_eq!(parse_maven_module(line), Some("[1/5] foo".to_string()));
    }

    #[test]
    fn parse_module_name_with_brackets() {
        let line = "[INFO] Building WildFly: Security [Legacy] 40.0.0.Beta1-SNAPSHOT [42/161]";
        assert_eq!(
            parse_maven_module(line),
            Some("[42/161] WildFly: Security [Legacy]".to_string())
        );
    }

    #[test]
    fn ignore_non_building_line() {
        let line = "[INFO] --- source:3.2.1:jar-no-fork (attach-sources) @ wildfly-parent ---";
        assert!(parse_maven_module(line).is_none());
    }

    #[test]
    fn ignore_empty_line() {
        assert!(parse_maven_module("").is_none());
    }

    #[test]
    fn ignore_info_line_without_building() {
        let line = "[INFO] Compiling 42 source files to /path/to/target/classes";
        assert!(parse_maven_module(line).is_none());
    }

    #[test]
    fn ignore_invalid_counter() {
        let line = "[INFO] Building foo 1.0 [abc/def]";
        assert!(parse_maven_module(line).is_none());
    }
}
