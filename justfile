watch +args='test':
  cargo watch --clear --exec '{{args}}'

open:
  open http://localhost:8000

outdated:
  cargo outdated -R

ci: clippy forbid
  cargo fmt -- --check
  cargo test --all
  cargo test --all -- --ignored

clippy:
  cargo clippy --all --all-targets -- --deny warnings

forbid:
  ./bin/forbid

serve:
  rm -rf tmp
  mkdir tmp
  cargo build
  ./target/debug/media package --root apps/comic --output tmp/app.package
  ./target/debug/media package --root content/comic --output tmp/content.package
  ./target/debug/media server --address 127.0.0.1:8000 --app tmp/app.package --content tmp/content.package
