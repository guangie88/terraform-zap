#![cfg_attr(feature = "cargo-clippy", deny(clippy))]
#![deny(missing_debug_implementations, warnings)]

//! # terraform-zap
//!
//! Script wrapper to perform finer terraform destroy

#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate failure;
extern crate is_executable;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate subprocess;
extern crate terraform_zap_ignore_lib;
extern crate toml;
extern crate which;
extern crate whiteread;
#[macro_use]
extern crate vlog;
extern crate yansi;

mod arg;
mod error;

use arg::Config;
use error::{Error, Result};
use failure::Fail;
use is_executable::IsExecutable;
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process;
use structopt::StructOpt;
use subprocess::{Exec, Redirection};
use terraform_zap_ignore_lib::Ignore;
use yansi::Paint;

const TF_CMD: &str = "terraform";
const TFIGNORE_FILE: &str = ".tfignore";

fn find_ignore() -> Result<Ignore> {
    let roots = {
        let mut cwd = env::current_dir()?;

        let mut roots = vec![];
        roots.push(cwd.clone());

        while cwd.pop() {
            roots.push(cwd.clone());
        }

        roots
    };

    let ignore_path = roots
        .into_iter()
        .map(|mut root| {
            root.push(PathBuf::from(TFIGNORE_FILE));
            root
        })
        .inspect(|root| v2!("Found .tfignore path: {:?}", root))
        .find(|ignore_path| Path::exists(ignore_path));

    if let Some(ignore_path) = ignore_path {
        let mut content = String::new();
        let mut f = File::open(ignore_path)?;
        f.read_to_string(&mut content)?;
        Ok(toml::from_str(&content)?)
    } else {
        Err(Error::MissingIgnore)?
    }
}

fn run(config: &Config) -> Result<()> {
    vlog::set_verbosity_level(config.verbose as usize);

    let tf_cmd = if let Some(ref tf_cmd) = config.tf_cmd {
        if !tf_cmd.is_executable() {
            Err(Error::NotExecutable)?
        }

        tf_cmd.clone()
    } else {
        which::which(TF_CMD)?
    };

    let state_capture = Exec::cmd(&tf_cmd)
        .args(&["state", "list"])
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Pipe)
        .capture()?;

    if !state_capture.exit_status.success() {
        Err(Error::CommandError(state_capture.stderr_str()))?
    }

    let targets_str = state_capture.stdout_str();
    let targets: Vec<String> = whiteread::parse_string(&targets_str)?;

    let ignore = find_ignore()?;

    let filtered_targets: Vec<String> = match ignore {
        Ignore::Exact { exact } => targets
            .into_iter()
            .filter(|target| !exact.iter().any(|resource| target == resource))
            .collect(),
    };

    let mut target_args = vec![];

    for filtered_target in filtered_targets {
        target_args.push("-target".to_owned());
        target_args.push(filtered_target)
    }

    let destroy_capture = Exec::cmd(&tf_cmd)
        .arg("destroy")
        .args(&target_args)
        .stdin(Redirection::None)
        .stderr(Redirection::Pipe)
        .capture()?;

    if !destroy_capture.exit_status.success() {
        Err(Error::CommandError(
            destroy_capture.stderr_str(),
        ))?
    }

    Ok(())
}

fn to_err_color<T>(arg: T) -> Paint<T>
where
    T: Display,
{
    Paint::new(arg).bold()
}

fn main() {
    let config = Config::from_args();

    match run(&config) {
        Ok(_) => v1!("terraform-zap completed!"),
        Err(e) => {
            ve1!(
                "{}",
                Paint::purple("terraform-zap encountered error!")
            );

            ve0!("{}", to_err_color(&e));

            if let Some(cause) = e.cause() {
                ve0!("Caused by: {}", to_err_color(cause));
            }

            if let Some(backtrace) = e.backtrace() {
                ve0!("Backtrace: {}", to_err_color(backtrace));
            }

            process::exit(1);
        }
    }
}
