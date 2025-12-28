alias f := format
alias fmt := format
alias ff := format_force

default:
  @just --list

format force="":
  #!/usr/bin/env nu
  let force = "{{force}}" == "-f" or "{{force}}" == "--force"
  let dirty = if $force { ["--allow-dirty"] } else { [] }

  cargo fmt
  cargo fix ...$dirty
  cargo clippy --fix ...$dirty

format_force: (format "-f")
