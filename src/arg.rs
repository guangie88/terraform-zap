#[derive(StructOpt, Debug)]
#[structopt(
    name = "terraform-zap",
    about = "Script wrapper to perform finer terraform destroy."
)]
pub struct Config {
    /// Path to `terraform` command. Can also use env var TF to override
    #[structopt(short = "c", long = "cmd")]
    pub tf_cmd: Option<String>,

    /// Verbose flag (-v, -vv, -vvv)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: u8,

    /// Additional arguments to pass to `terraform destroy`
    pub pass_args: Vec<String>,
}
