# `terraform-zap`

[![Build Status](https://travis-ci.org/guangie88/terraform-zap.svg?branch=master)](https://travis-ci.org/guangie88/terraform-zap)
[![codecov](https://codecov.io/gh/guangie88/terraform-zap/branch/master/graph/badge.svg)](https://codecov.io/gh/guangie88/terraform-zap)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Script wrapper to perform finer terraform destroy. This means that `terraform`
must still be installed and residing within `PATH` environment variable.

Currently if any of the `.tf` files contain `prevent_destroy = true` for any of
the resources, `terraform destroy` will fail and there is no flag to force
`terraform` to skip all resources. This script wrapper helps to alleviate the
issue by parsing `.tfignore` file in the current working directory, where the
`.tf` files are residing in.

## Example `.tfignore` format

```toml
exact = [
    "important_database.xxx",
    "important_role.xxx",
]
```

If there are resources that exactly match the names above, these resources are
automatically skipped, solving the problem of having to type complicated
commands in order to skip the above resources to possibly resolve the
`prevent_destroy = true` issue, with just a single command.

## How to install

Use [https://www.rustup.rs/](`rustup`) and get the commands `cargo` and
`rustc`.

Run `cargo install terraform-zap` for the installation. This will automatically
fetch `terraform-zap` CLI application from [`crates.io`](https://crates.io/)
and install into your Cargo installation binary directory.

## How to run

With `.tfignore` file in place, simply run `terraform-zap`. You should see
mainly `terraform destroy` logs in place, but the ignored resources should now
no longer appear during the confirmation.

If previously there were resources
with `prevent_destroy = true` set, if these resources are correctly ignored,
the confirmation prompt should appear properly.

For more CLI argument details, type `terraform-zap -h`.

## How to build (not necessary for users)

Run `cargo build --all --release`. This builds all the associated libraries
and the executable. The executable will be built in
`target/release/terraform-zap`.
