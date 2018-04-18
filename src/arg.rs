use std::path::PathBuf;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "terraform-zap",
    about = "Script wrapper to perform finer terraform destroy."
)]
pub struct Config {
    #[structopt(short = "c", long = "cmdpath", parse(from_os_str))]
    /// Path to `terraform` command (optional)
    pub tf_cmd: Option<PathBuf>,

    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    /// Verbose flag (-v, -vv, -vvv)
    pub verbose: u8,

    #[structopt(short = "p", long = "pass")]
    /// Additional arguments to pass to `terraform destroy`
    pub pass_args: Option<String>,
}
