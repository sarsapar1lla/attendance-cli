#!/bin/sh

mkdir docs
cargo run spec > ./docs/attendance.usage.kdl
usage generate markdown --file ./docs/attendance.usage.kdl --out-file ./docs/attendance.md 
