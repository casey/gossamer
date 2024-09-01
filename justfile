watch +args='test':
  cargo watch --clear --exec '{{args}}'

outdated:
  cargo outdated

unused:
  cargo +nightly udeps --all-targets

ci: clippy forbid
  cargo fmt -- --check
  cargo test
  cargo test -- --ignored

clippy:
  cargo clippy --all-targets -- --deny warnings

fmt:
  cargo fmt

forbid:
  ./bin/forbid

open:
  open http://localhost

serve port="80":
  cargo build
  target/debug/gossamer package --root tests/packages/comic --output build/test-comic.package
  target/debug/gossamer server \
    --http-port {{port}} \
    --packages \
      build/test-comic.package

update-modern-normalize:
  curl \
    https://raw.githubusercontent.com/sindresorhus/modern-normalize/main/modern-normalize.css \
    > static/modern-normalize.css
