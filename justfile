import '~/lib/justfile'

dump_src_:
  'ls' ./Cargo.toml ./README.md src/main.rs  | xargs bat --style=full

dump_src:
  just dump_src_ > src_dump.txt

debug_dump_src:
  just dump_src
  bat ./src_dump.txt

# Update the README.md file with the contents of the main.rs file
readme:
  rg '//!' src/main.rs | sd '^//! ' '' | sd '^//!$' '' > README.md
  echo '' >> README.md
  echo '<footer> This README file is generated based on the docs in `src/main.rs`. </footer>' >> README.md
