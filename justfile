import '~/lib/justfile'

# Update the README.md file with the contents of the main.rs file
readme:
  rg '//!' src/main.rs | sd '^//! ' '' | sd '^//!$' '' > README.md
  echo '' >> README.md
  echo '##' >> README.md
  echo 'This README file is generated based on the docs in `src/main.rs`.' >> README.md

test:
  cargo test

fmt:
  cargo +nightly fmt

watch recipe *args:
  watchexec --clear -e rs -- just {{ recipe }} {{ args }}

split_and_watch arg *args:
  just split_and_run just watch {{ arg }} {{ args }}

watchers:
  just split_and_watch readme
  just split_and_watch test
  just split_and_watch fmt

dump_src_:
  'ls' ./Cargo.toml ./README.md src/main.rs  | xargs bat --style=full

dump_src:
  just dump_src_ > src_dump.txt

debug_dump_src:
  just dump_src
  bat ./src_dump.txt
