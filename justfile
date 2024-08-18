watch +args='test':
  cargo watch --clear --exec '{{args}}'

outdated:
  cargo outdated -R

# todo:
# - add unused deps recipe

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

open:
  open http://localhost

serve port="80":
  cargo build
  target/debug/gossamer package --root tests/packages/comic --output build/test-comic.package
  RUST_LOG=gossamer=trace target/debug/gossamer server \
    --http-port {{port}} \
    --packages \
      build/test-comic.package

update-modern-normalize:
  curl \
    https://raw.githubusercontent.com/sindresorhus/modern-normalize/main/modern-normalize.css \
    > static/modern-normalize.css
