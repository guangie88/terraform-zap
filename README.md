# `terraform-zap`

[![Build Status](https://travis-ci.org/guangie88/terraform-zap.svg?branch=master)](https://travis-ci.org/guangie88/terraform-zap)
[![codecov](https://codecov.io/gh/guangie88/terraform-zap/branch/master/graph/badge.svg)](https://codecov.io/gh/guangie88/terraform-zap)
[![Crates.io](https://img.shields.io/crates/v/terraform-zap.svg)](https://crates.io/crates/terraform-zap)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Script wrapper to perform finer terraform destroy. This means that `terraform`
must still be installed and residing within `PATH` environment variable.

Currently if any of the `.tf` files contain `prevent_destroy = true` for any of
the resources, `terraform destroy` will fail and there is no flag to force
`terraform` to skip all resources. This script wrapper helps to alleviate the
issue by parsing `.tfzignore` file in the current working directory, where the
`.tf` files are residing in.

Previously the ignore file was named `.tfignore` up to `v0.1.1`, however this
clashes with the Microsoft Team Foundation ignore file naming, so this has been
changed to `.tfzignore`.

## Example `.tfzignore` format

```toml
exact = [
    "postgresql_database.some_db_name",
    "postgresql_role.some_role_name",
]
```

If there are resources that exactly match the names above, these resources are
automatically skipped, solving the problem of having to type complicated
commands in order to skip the above resources to possibly resolve the
`prevent_destroy = true` issue, with just a single command.

## How to install

### Direct download for Linux (AMD64)

The easiest way is to run the install script using shell as shown below.

```bash
curl -sSf https://raw.githubusercontent.com/guangie88/terraform-zap/master/install-index.sh | sudo sh
```

You will need to run as `root`, or run via `sudo`, since the script will place
`terraform-zap` binary file into `/usr/local/bin/`.

You may also choose to visit
[releases](https://github.com/guangie88/terraform-zap/releases)
and download the latest version of statically built binary in the zip asset.

### Via `cargo install`

You will first need to install `cargo` from <https://rustup.rs/>. The
installation process should be very straightforward for any major architecture
and operating system.

After which, run `cargo install terraform-zap` for the installation. This will
automatically fetch `terraform-zap` CLI application from
[`crates.io`](https://crates.io/), compile and install into your Cargo
installation binary directory.

## How to run

With `.tfzignore` file in place, simply run `terraform-zap`. You should see
mainly `terraform destroy` logs in place, but the ignored resources should now
no longer appear during the confirmation.

If previously there were resources
with `prevent_destroy = true` set, if these resources are correctly ignored,
the confirmation prompt should appear properly.

For more CLI argument details, type `terraform-zap -h`.

## `terraform zap` instead of `terraform-zap`

It is possible to allow `terraform zap [...]` to run `terraform-zap [...]`, by
using a function that is exported on startup. This makes the external program
look like part of a `terraform` subcommand, but obviously this is completely
optional.

Add the following `bash` function to the any of your startup script (e.g.
`~/.bashrc`), to allow the above

```bash
terraform() {
    if [[ $1 == "zap" ]]; then
        command terraform-zap "${@:2}"
    else
        command terraform "${@:1}"
    fi
}
```

Either restart the current terminal, or run `source ~/.bashrc` (if following
the example), and try `terraform zap` to check if the above function is working
as intended.

## How to build (not necessary for users)

Run `cargo build --all --release`. This builds all the associated libraries
and the executable. The executable will be built in
`target/release/terraform-zap`.

## Contributions

Pull requests are welcome to facilitate improvements to the repository.

Thanks to [`@chrissng`](https://github.com/chrissng) for providing the original
`terraform destroy` command that only targets non-protected resources. The
original command line is as follow:

```bash
TARGETS=$(for I in $(terraform state list | grep -v postgresql); \
    do echo " -target $I"; done); \
    echo terraform destroy $TARGETS
```
