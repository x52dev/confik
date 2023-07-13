_list:
    @just --list

check:
    just --unstable --fmt --check
    npx -y prettier --check '**/*.md'
    taplo lint

fmt:
    just --unstable --fmt
    npx -y prettier --write '**/*.md'
    taplo format
