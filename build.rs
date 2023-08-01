use clap::CommandFactory;
use clap_complete::{generate_to, shells::*};
use std::{env, fs, path::PathBuf, process};

include!("src/bin/blaze/cli.rs");

fn main() {
    let mut cmd = Cli::command();
    let app_name = cmd.get_name().to_string();

    let completions_dir = "completions";
    create_dir(completions_dir);

    if let Err(e) = generate_completions(&mut cmd, &app_name, completions_dir) {
        panic!("Error: {}", e);
    }

    let manpage_dir = "man";
    create_dir(manpage_dir);

    if let Err(e) = generate_manpages(cmd, manpage_dir) {
        panic!("Error: {}", e);
    }

    let build_timestamp: String = get_buildtimestamp();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);
    println!("cargo:rustc-env=BUILD_VERSION={}", get_version());
}

fn generate_manpages(cmd: clap::Command, out_dir: &str) -> std::io::Result<()> {
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer).unwrap();

    std::fs::write(PathBuf::from(out_dir).join("blaze.1"), buffer)?;

    Ok(())
}

fn generate_completions(
    cmd: &mut clap::Command,
    app_name: &str,
    outdir: &str,
) -> std::io::Result<()> {
    generate_to(Bash, cmd, app_name, outdir)?;
    generate_to(Zsh, cmd, app_name, outdir)?;
    generate_to(Fish, cmd, app_name, outdir)?;
    generate_to(PowerShell, cmd, app_name, outdir)?;

    Ok(())
}

fn get_version() -> String {
    let mut version_env: String = "".to_string();
    let mut cmd = process::Command::new("git");

    cmd.arg("describe");
    cmd.arg("--abbrev=0");
    cmd.arg("--tags=0");

    match cmd.status() {
        Ok(status) => {
            if status.success() {
                if let Ok(output) = cmd.output() {
                    return format!("{:?}", output);
                }
            } else {
                println!("Error: Cannot get build version using `git` using CARGO_PKG_VERSION");
                version_env = env!("CARGO_PKG_VERSION").to_string();
            }
        }
        Err(e) => {
            panic!("Error: Failed getting build version with git: {}", e);
        }
    }

    version_env
}

fn get_buildtimestamp() -> String {
    return chrono::Local::now()
        .naive_local()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
}

fn create_dir(dir_name: &str) {
    if let Err(e) = fs::create_dir_all(dir_name) {
        panic!("Error: {}", e);
    }
}
