set unstable

mod library 'crates/library'

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

clean:
  cargo clean
  rm -rf tmp
  rm -rf crates/library/build
  rm -rf crates/library/library.package

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
  just library package
  cargo build
  mkdir -p target/packages
  target/debug/gossamer package --root tests/packages/app-comic --output target/packages/app.package
  target/debug/gossamer package --root tests/packages/comic --output target/packages/comic.package
  target/debug/gossamer server \
    --open \
    --address 127.0.0.1:8000 \
    --packages crates/library/library.package target/packages/app.package target/packages/comic.package
