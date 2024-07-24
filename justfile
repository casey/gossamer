watch +args='test':
  cargo watch --clear --exec '{{args}}'

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
  rm -rf build

open:
  open http://localhost

serve: build (package 'app-viewer') (package 'comic-viewer') (package 'library-viewer')
  mkdir -p target/packages
  target/debug/gossamer package --root tests/packages/comic --output build/test-comic.package
  target/debug/gossamer server \
    --http-port 8000 \
    --bootstrap 1e24b3c62cdb105a975ed71f934abd174bb6de6258e3799b3d510ff17b62d768@127.0.0.1:57423 \
    --packages \
      build/app-viewer.package \
      build/comic-viewer.package \
      build/library-viewer.package \
      build/test-comic.package

package crate: build files
  rm -rf build/{{crate}}
  mkdir -p build/{{crate}}
  cargo build \
    --package {{crate}} \
    --target wasm32-unknown-unknown
  cp target/wasm32-unknown-unknown/debug/{{crate}}.wasm build/{{crate}}/index.wasm
  wasm-bindgen \
    --target web \
    --no-typescript \
    build/{{crate}}/index.wasm \
    --out-dir build/{{crate}}
  mv build/{{crate}}/index_bg.wasm build/{{crate}}/index.wasm
  mv build/{{crate}}/index.js build/{{crate}}/loader.js
  rsync -aqvz --exclude .DS_Store files/ build/files/ crates/{{crate}}/files/ build/{{crate}}/
  target/debug/gossamer package --root build/{{crate}} --output build/{{crate}}.package

files:
  test -f build/files/modern-normalize.css || just update-files

update-files:
  mkdir -p build/files
  curl \
    https://raw.githubusercontent.com/sindresorhus/modern-normalize/main/modern-normalize.css \
    > build/files/modern-normalize.css
