set unstable

mod root 'crates/root'

watch +args='test':
  cargo watch --clear --exec '{{args}}'

open:
  open http://localhost:8000

outdated:
  cargo outdated -R

ci: clippy forbid
  cargo fmt --all -- --check
  cargo test --all
  cargo test --all -- --ignored

clippy:
  cargo clippy --all --all-targets -- --deny warnings

fmt:
  cargo fmt --all

forbid:
  ./bin/forbid

build:
  cargo build

serve: build
  rm -rf tmp
  mkdir tmp
  target/debug/gossamer package --root tests/packages/app-comic --output tmp/app.package
  target/debug/gossamer package --root tests/packages/app-root --output tmp/root.package
  target/debug/gossamer package --root tests/packages/comic --output tmp/comic.package
  target/debug/gossamer server \
    --open \
    --address 127.0.0.1:8000 \
    --packages tmp/root.package tmp/app.package tmp/comic.package

apps:
  just root package
  cargo build
  target/debug/gossamer package --root tests/packages/app-comic --output tmp/app.package
  target/debug/gossamer package --root tests/packages/comic --output tmp/comic.package
  target/debug/gossamer server \
    --open \
    --address 127.0.0.1:8000 \
    --packages crates/root/root.package tmp/app.package tmp/comic.package
