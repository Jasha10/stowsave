import '~/lib/justfile'

dump_src_:
  'ls' ./Cargo.toml ./README.md src/main.rs  | xargs bat --style=full

dump_src:
  just dump_src_ > src_dump.txt

debug_dump_src:
  just dump_src
  bat ./src_dump.txt
