#!/bin/sh

set -e

cd example
 ~/.cargo/bin/wasm-pack build --target web
cargo run --package clap_web_code_gen --bin clap-web-gen

rsync -av --exclude .git --exclude .gitignore pkg/ ../../clap-web-gen-web-example-release

cd ../../clap-web-gen-web-example-release
git add -A .
git commit -m 'new release'
git push origin master
