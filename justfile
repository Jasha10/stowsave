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

## RELEASE

# The publish.yaml workflow runs `cargo publish` automatically when the tag lands on origin.
# Usage:  just release 0.1.7
# Cut a release: bump version, run tests, commit, tag, and (after confirmation) push.
release VERSION:
  #!/usr/bin/env bash
  # The shebang MUST be the first line of the recipe body — `just` only treats this
  # as a single-shell recipe when `#!` is the first character. Comments above it
  # break that detection and cause each line to run in a fresh `sh`.
  set -euo pipefail
  VERSION="{{VERSION}}"

  # ---- Pre-flight checks (all run before any state changes) ----

  # 1. Version must be N.N.N — matches the publish.yaml tag trigger and the project's
  #    recent tag style (0.1.5, 0.1.6). We don't allow the legacy `v*` form.
  [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] \
    || { echo "VERSION must be N.N.N (got: $VERSION)"; exit 1; }

  # 2. Working tree must be clean. Otherwise the release commit would silently pick up
  #    unrelated edits — especially risky in this repo, which gets touched interactively.
  [[ -z "$(git status --porcelain)" ]] \
    || { echo "Working tree not clean — commit or stash first."; exit 1; }

  # 3. Must be on main. publish.yaml doesn't enforce this, but project convention does.
  branch="$(git rev-parse --abbrev-ref HEAD)"
  [[ "$branch" == "main" ]] \
    || { echo "Not on main (on: $branch). Releases ship from main."; exit 1; }

  # 4. Tag must not already exist locally. crates.io versions are immutable, so catching
  #    a collision here beats discovering it after pushing.
  git rev-parse "$VERSION" >/dev/null 2>&1 \
    && { echo "Tag $VERSION already exists."; exit 1; } || true

  # 5. New version must differ from the current one in Cargo.toml. `sd -p` prints the
  #    captured group; `rg -m1` filters to the first semver-looking line so we ignore
  #    `[dev-dependencies] version = "..."` lines below.
  current="$(sd -p '^version = "([^"]+)"' '$1' < Cargo.toml | rg -m1 '^[0-9]+\.[0-9]+\.[0-9]+$' || true)"
  [[ "$current" != "$VERSION" ]] \
    || { echo "Version is already $VERSION."; exit 1; }

  # ---- Mutations begin here ----

  # Bump the [package] version line in Cargo.toml.
  sd '^version = "[^"]+"' "version = \"$VERSION\"" Cargo.toml

  # Refresh Cargo.lock so it matches the new Cargo.toml. The CI publish step uses
  # --locked and will fail if the lockfile is out of sync. The first attempt keeps
  # Cargo.lock honest; the fallback regenerates it if the version bump itself made
  # --locked fail.
  cargo build --locked || cargo build

  # Run tests before producing a tag — a bad tag can't be retracted from crates.io.
  cargo test

  # Stage only the two files we changed. Explicit paths, no `git add -A`.
  git add Cargo.toml Cargo.lock

  # Commit message = bare version string. Matches the project's commit log style.
  git commit -m "$VERSION"

  # Annotated tag (-a). Required so `git push --follow-tags` carries it — lightweight
  # tags are skipped by --follow-tags.
  git tag -a "$VERSION" -m "$VERSION"

  # ---- Confirmation gate before push ----

  echo
  echo "Local release prepared:"
  git --no-pager log -1 --oneline
  echo "Tag: $VERSION"
  echo
  read -r -p "Push to origin now? [y/N] " ans
  [[ "$ans" == "y" || "$ans" == "Y" ]] \
    || { echo "Skipping push. To finish later: git push --follow-tags"; exit 0; }

  # Single push carries the commit + the annotated tag, which is what fires publish.yaml.
  git push --follow-tags
  echo "Pushed. Watch the publish job: gh run watch --workflow=publish.yaml"

# Usage:  just release-patch
# Auto-bumps the patch component of the current Cargo.toml version, then delegates to `release`.
release-patch:
  #!/usr/bin/env bash
  set -euo pipefail
  # Parse the current [package] version. `sd -p` prints the captured group; `rg -m1`
  # filters to the first semver-shaped line so we ignore any `version = "..."` entries
  # under [dependencies]/[dev-dependencies] further down the file.
  current="$(sd -p '^version = "([^"]+)"' '$1' < Cargo.toml | rg -m1 '^[0-9]+\.[0-9]+\.[0-9]+$')"
  IFS='.' read -r major minor patch <<< "$current"
  next="$major.$minor.$((patch + 1))"
  echo "Current: $current  →  Next patch: $next"
  # Delegate to `release` for all the actual work (validation, build, test, commit,
  # tag, confirmation prompt, push). Keeping this recipe a pure wrapper means there
  # is exactly one code path for cutting a release.
  just release "$next"

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
