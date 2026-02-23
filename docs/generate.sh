#!/bin/sh

cargo run spec > ./docs/attendance.usage.kdl
usage generate markdown --file ./docs/attendance.usage.kdl --out-dir docs --multi 
