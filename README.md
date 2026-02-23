# attendance-cli

[![Run tests](https://github.com/sarsapar1lla/attendance-cli/actions/workflows/workflow.yaml/badge.svg)](https://github.com/sarsapar1lla/attendance-cli/actions/workflows/workflow.yaml)

Attendance is a simple CLI for tracking your office attendance.

## Installation

Install using `cargo` direct from GitHub:

```shell
$ cargo install --git https://github.com/sarsapar1lla/attendance-cli --locked
```

## Basic Commands

> See the [docs](./docs/attendance.md) for details on all available commands and options.

### Log attendance

Log your office attendance using:

```shell
$ attendance log
```

Exclusions can be logged using the `--type` option:

```shell
$ attendance log --type wfh
```

To append or delete a record, use the `--mode` option.

### Summarise attendance

Summarise your attendance for the current month using:

```shell
$ attendance summary
```

To view summaries of the last `x` months, use:

```shell
$ attendance summary --months x
```

### View the log

View logged entries using:

```shell
$ attendance show
```

To view the top `x` records, use the `--top` option.

### Generate shell completions

Generate shell completions using:

```shell
$ attendance completion bash > ~/.local/share/bash-completion/completions/attendance
$ attendance completion fish > ~/.config/fish/completions/attendance.fish
$ attendance completion zsh > /usr/local/share/zsh/site-functions/_attendance
```

## Local Development

Build the project locally using:

```shell
$ cargo build
```

To run all tests, use:

```shell
$ cargo test
```

### Pre-commit

This project uses [`prek`](https://github.com/j178/prek) to run pre-commit hooks.

To install hooks locally, use:

```shell
$ prek install
```

CLI docs are automatically generated using [`usage`](https://usage.jdx.dev/). Follow the
installation guide [here](https://usage.jdx.dev/cli/#installation).
