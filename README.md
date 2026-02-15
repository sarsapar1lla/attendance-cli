# attendance-cli

CLI for tracking office attendance

[![Run tests](https://github.com/sarsapar1lla/attendance-cli/actions/workflows/workflow.yaml/badge.svg)](https://github.com/sarsapar1lla/attendance-cli/actions/workflows/workflow.yaml)

## Installation

Install using `cargo` direct from GitHub:

```shell
$ cargo install --git https://github.com/sarsapar1lla/attendance-cli --locked
```

## Basic Commands

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

> See `attendance log --help` for all available options.

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

## Local Development

Build the project locally using:

```shell
$ cargo build
```

To run all tests, use:

```shell
$ cargo test
```
