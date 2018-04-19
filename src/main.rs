#![cfg_attr(feature = "cargo-clippy", deny(clippy))]
// #![deny(missing_debug_implementations, warnings)]

//! # terraform-zap
//!
//! Script wrapper to perform finer terraform destroy. This means that
//! `terraform` must still be installed and residing within `PATH` environment
//! variable.
//!
//! Currently if any of the `.tf` files contain `prevent_destroy = true` for any
//! of the resources, `terraform destroy` will fail and there is no flag to
//! force `terraform` to skip all resources. This script wrapper helps to
//! alleviate the issue by parsing `.tfzignore` file in the current working
//! directory, where the `.tf` files are residing in.

#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate failure;
extern crate is_executable;
extern crate itertools;
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
use error::{CommandErrorCapture, Error, Result};
use failure::Fail;
use is_executable::IsExecutable;
use itertools::Itertools;
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::iter;
use std::path::{Path, PathBuf};
use std::process;
use structopt::StructOpt;
use subprocess::{Exec, ExitStatus, Redirection};
use terraform_zap_ignore_lib::Ignore;
use yansi::Paint;

const TF_CMD: &str = "terraform";
const TFZIGNORE_FILE: &str = ".tfzignore";
const OTHER_ERROR_EXIT_CODE: i32 = 1;
const UNDETERMINED_EXIT_CODE: i32 = 2;

fn find_ignore(mut cwd: PathBuf) -> Result<Option<Ignore>> {
    let ignore_path = {
        cwd.push(TFZIGNORE_FILE);
        cwd
    };

    if Path::exists(&ignore_path) {
        v2!("Found .tfzignore path: {:?}", ignore_path);

        let mut content = String::new();
        let mut f = File::open(ignore_path)?;
        f.read_to_string(&mut content)?;
        Ok(Some(toml::from_str(&content)?))
    } else {
        v2!(
            "{:?} is missing, no filtering is performed...",
            ignore_path
        );

        Ok(None)
    }
}

fn interleave_targets(targets: &[String]) -> Vec<&str> {
    let filtered_target_refs = targets.iter().map(|t| t.as_ref());

    iter::repeat("-target")
        .take(targets.len())
        .interleave(filtered_target_refs)
        .collect()
}

fn run(config: &Config) -> Result<()> {
    vlog::set_verbosity_level(config.verbose as usize);

    // set-up for Windows
    if cfg!(windows) && !Paint::enable_windows_ascii() {
        Paint::disable();
    }

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
        Err(CommandErrorCapture::new(
            state_capture.exit_status,
            state_capture.stderr_str(),
        ))?
    }

    let targets_str = state_capture.stdout_str();
    let targets: Vec<String> = whiteread::parse_string(&targets_str)?;

    // ignore file is allowed to be missing
    let ignore = find_ignore(env::current_dir()?)?;

    let filtered_targets: Vec<String> = if let Some(ignore) = ignore {
        match ignore {
            Ignore::Exact { exact } => targets
                .into_iter()
                .filter(|target| {
                    !exact.iter().any(|resource| target == resource)
                })
                .collect(),
        }
    } else {
        // missing ignore file means no filtering is done
        targets
    };

    let target_args = interleave_targets(&filtered_targets);

    let destroy_capture = Exec::cmd(&tf_cmd)
        .arg("destroy")
        .args(&target_args)
        .args(&config.pass_args)
        .stdin(Redirection::None)
        .stderr(Redirection::Pipe)
        .capture()?;

    if !destroy_capture.exit_status.success() {
        Err(CommandErrorCapture::new(
            destroy_capture.exit_status,
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

fn exit_status_to_code(exit_status: &ExitStatus) -> i32 {
    match *exit_status {
        // this is not exactly lossy, but will wrap (u32 -> i32)
        ExitStatus::Exited(code) => code as i32,
        ExitStatus::Signaled(code) => i32::from(code),
        ExitStatus::Other(code) => code,
        ExitStatus::Undetermined => UNDETERMINED_EXIT_CODE,
    }
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

            match e {
                Error::CommandError(ref capture) => {
                    process::exit(exit_status_to_code(&capture.exit_status))
                }

                _ => process::exit(OTHER_ERROR_EXIT_CODE),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs::{create_dir, remove_dir, remove_file};
    use std::io::Write;
    use std::{i32, u32, u8};

    struct IgnoreDirSetup {
        dir: PathBuf,
        ignore_path: Option<PathBuf>,
    }

    impl IgnoreDirSetup {
        fn new(sub_dir: &str, content: Option<&str>) -> IgnoreDirSetup {
            IgnoreDirSetup::new_with_custom_tmp_dir(
                temp_dir(),
                sub_dir,
                content,
            )
        }

        fn new_with_custom_tmp_dir(
            tmp_dir: PathBuf,
            sub_dir: &str,
            content: Option<&str>,
        ) -> IgnoreDirSetup {
            let mut dir = tmp_dir;
            dir.push(sub_dir);
            create_dir(&dir).unwrap();

            let ignore_path = if let Some(content) = content {
                let mut ignore_path = dir.clone();
                ignore_path.push(TFZIGNORE_FILE);

                let mut ignore_file = File::create(&ignore_path).unwrap();

                ignore_file
                    .write_fmt(format_args!("{}", content))
                    .unwrap();

                ignore_file.sync_all().unwrap();

                Some(ignore_path)
            } else {
                None
            };

            IgnoreDirSetup {
                dir,
                ignore_path,
            }
        }
    }

    impl Drop for IgnoreDirSetup {
        fn drop(&mut self) {
            if let Some(ref ignore_path) = self.ignore_path {
                remove_file(ignore_path).unwrap();
            }

            if Path::exists(&self.dir) {
                remove_dir(&self.dir).unwrap();
            }
        }
    }

    #[test]
    fn test_find_ignore_valid_1() {
        const IGNORE_CONTENT: &str = r#"
            exact = []
        "#;

        let setup = IgnoreDirSetup::new(
            "terraform-zap-test_find_ignore_valid_1",
            Some(IGNORE_CONTENT),
        );

        let ignore = find_ignore(setup.dir.clone()).unwrap().unwrap();

        match ignore {
            Ignore::Exact { exact } => {
                assert!(exact.len() == 0);
            }
        }
    }

    #[test]
    fn test_find_ignore_valid_2() {
        const IGNORE_CONTENT: &str = r#"
            exact = [
                "a.b",
                "x.y",
            ]
        "#;

        let setup = IgnoreDirSetup::new(
            "terraform-zap-test_find_ignore_valid_2",
            Some(IGNORE_CONTENT),
        );

        let ignore = find_ignore(setup.dir.clone()).unwrap().unwrap();

        match ignore {
            Ignore::Exact { exact } => {
                assert!(exact.len() == 2);
                assert_eq!("a.b", exact[0]);
                assert_eq!("x.y", exact[1]);
            }
        }
    }

    #[test]
    fn test_find_ignore_valid_3() {
        let setup =
            IgnoreDirSetup::new("terraform-zap-test_find_ignore_valid_3", None);

        let ignore = find_ignore(setup.dir.clone()).unwrap();

        assert!(ignore.is_none());
    }

    #[test]
    fn test_find_ignore_valid_4() {
        const IGNORE_CONTENT: &str = r#"
            exact = [
                "a.b",
                "x.y",
            ]
        "#;

        let base_setup = IgnoreDirSetup::new(
            "terraform-zap-test_find_ignore_valid_4",
            Some(IGNORE_CONTENT),
        );

        let actual_setup = IgnoreDirSetup::new_with_custom_tmp_dir(
            base_setup.dir.clone(),
            "sub",
            None,
        );

        let ignore = find_ignore(actual_setup.dir.clone()).unwrap();
        assert!(ignore.is_none());
    }

    #[test]
    fn test_find_ignore_invalid_1() {
        const IGNORE_CONTENT: &str = "";

        let setup = IgnoreDirSetup::new(
            "terraform-zap-test_find_ignore_invalid_1",
            Some(IGNORE_CONTENT),
        );

        let ignore = find_ignore(setup.dir.clone());
        assert!(ignore.is_err());
    }

    #[test]
    fn test_find_ignore_invalid_2() {
        const IGNORE_CONTENT: &str = "[]";

        let setup = IgnoreDirSetup::new(
            "terraform-zap-test_find_ignore_invalid_2",
            Some(IGNORE_CONTENT),
        );

        let ignore = find_ignore(setup.dir.clone());
        assert!(ignore.is_err());
    }

    #[test]
    fn test_find_ignore_invalid_3() {
        const IGNORE_CONTENT: &str = r#"
            exact = abc [
                "a.b",
            ]
        "#;

        let setup = IgnoreDirSetup::new(
            "terraform-zap-test_find_ignore_invalid_3",
            Some(IGNORE_CONTENT),
        );

        let ignore = find_ignore(setup.dir.clone());
        assert!(ignore.is_err());
    }

    #[test]
    fn test_target_interleave_1() {
        let targets = vec![];
        let reference: [&str; 0] = [];

        assert_eq!(
            &reference,
            interleave_targets(&targets).as_slice()
        );
    }

    #[test]
    fn test_target_interleave_2() {
        let targets = vec!["A".to_owned()];

        assert_eq!(
            &["-target", "A"],
            interleave_targets(&targets).as_slice()
        );
    }

    #[test]
    fn test_target_interleave_3() {
        let targets = vec!["A".to_owned(), "B".to_owned(), "C".to_owned()];

        assert_eq!(
            &["-target", "A", "-target", "B", "-target", "C"],
            interleave_targets(&targets).as_slice()
        );
    }

    #[test]
    fn test_exit_status_to_code_1() {
        let code = exit_status_to_code(&ExitStatus::Exited(u32::MIN));
        assert_eq!(0, code);
    }

    #[test]
    fn test_exit_status_to_code_2() {
        // overflow wrap
        let code = exit_status_to_code(&ExitStatus::Exited(u32::MAX));
        assert_eq!(-1, code);
    }

    #[test]
    fn test_exit_status_to_code_3() {
        // overflow wrap
        let code = exit_status_to_code(&ExitStatus::Signaled(u8::MIN));
        assert_eq!(u8::MIN as i32, code);
    }

    #[test]
    fn test_exit_status_to_code_4() {
        let code = exit_status_to_code(&ExitStatus::Signaled(u8::MAX));
        assert_eq!(u8::MAX as i32, code);
    }

    #[test]
    fn test_exit_status_to_code_5() {
        let code = exit_status_to_code(&ExitStatus::Other(i32::MIN));
        assert_eq!(i32::MIN, code);
    }

    #[test]
    fn test_exit_status_to_code_6() {
        let code = exit_status_to_code(&ExitStatus::Other(i32::MAX));
        assert_eq!(i32::MAX, code);
    }

    #[test]
    fn test_exit_status_to_code_7() {
        let code = exit_status_to_code(&ExitStatus::Undetermined);
        assert_eq!(UNDETERMINED_EXIT_CODE, code);
    }
}
