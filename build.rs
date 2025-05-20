use fs::{read_to_string, write};
use std::path::Path;
use std::{env, fs};

use anyhow::{Context, Result};
use clap_complete::generate_to;
use clap_complete::shells::{Bash, Elvish, Fish, PowerShell, Zsh};
use diffy::{Patch, apply};

include!("src/app.rs");

const APP_NAME: &str = "wado";

fn main() -> Result<()> {
    generate_shell_completions()?;
    Ok(())
}

fn generate_shell_completions() -> Result<()> {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").with_context(|| "CARGO_MANIFEST_DIR not set")?;
    let manifest_dir = Path::new(manifest_dir.as_str()).join("completions/");
    let mut app = build_app();
    app.set_bin_name(APP_NAME);

    generate_to(Fish, &mut app, APP_NAME, &manifest_dir)?;
    generate_to(Bash, &mut app, APP_NAME, &manifest_dir)?;
    generate_to(Zsh, &mut app, APP_NAME, &manifest_dir)?;
    generate_to(PowerShell, &mut app, APP_NAME, &manifest_dir)?;
    generate_to(Elvish, &mut app, APP_NAME, &manifest_dir)?;

    #[cfg(not(windows))]
    {
        patch(&manifest_dir, "wado.fish")?;
        patch(&manifest_dir, "wado.bash")?;
        patch(&manifest_dir, "_wado")?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn patch(manifest_dir: &Path, completions: &str) -> Result<()> {
    let completions_path_buf = manifest_dir.join(completions);
    let completions_path = completions_path_buf.as_path();
    let completions_str = read_to_string(completions_path)?;
    let patch_str = read_to_string(manifest_dir.join(format!("{}.diff", completions)))?;
    let patch = Patch::from_str(patch_str.as_str())?;
    let patched = apply(completions_str.as_str(), &patch)?;
    write(completions_path, patched)?;
    Ok(())
}
