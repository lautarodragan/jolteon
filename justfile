alias f := fmt
alias ff := fmt_force

fmt:
  cargo fmt
  cargo fix
  cargo clippy --fix

fmt_force:
  cargo fmt
  cargo fix --allow-dirty
  cargo clippy --fix --allow-dirty
