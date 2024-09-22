import '~/lib/justfile'

## THINGS TO WATCH

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

## UTILS:

watchers:
  just split_and_watch readme
  just split_and_watch test
  just split_and_watch fmt

## FOR DUMPING SRC FILES SO AI CAN READ THEM

dump_src_:
  'ls' ./Cargo.toml ./README.md src/main.rs  | xargs bat --style=full

dump_src:
  just dump_src_ > src_dump.txt

debug_dump_src:
  just dump_src
  bat ./src_dump.txt
