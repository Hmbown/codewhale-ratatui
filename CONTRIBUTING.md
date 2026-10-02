# Contributing

Thanks for helping. This crate is the terminal half of one design language
shared with the Codewhale desktop app and website, so most changes are either a
component or a token/whale update.

## Set up

```sh
cargo build
cargo test                       # unit, generated-roles, whale and snapshot tests
cargo run --example gallery      # look at every component; p changes the profile
```

The toolchain is pinned in `rust-toolchain.toml`.

## Before you open a PR

- Run `cargo test`. If a snapshot changes on purpose, review it with
  `cargo insta review` and say why in the PR.
- Paint from design roles, never raw colors. Snapshots record the role each cell
  was painted with, so painting the wrong role fails them.
- Pair every state with a mark and a word; nothing may depend on color alone.
- Check the change in the gallery profiles that matter to it (`no-color`,
  `ascii` and `ansi-16` catch most regressions).
- Token and whale data are generated: use the commands in the README under
  "Update the tokens" and "Update the whale" instead of editing `src/roles.rs`
  or `assets/whale-v2.scenes` by hand.

## Pull requests

Keep a PR to one change, with a short before/after description. Commits use a
conventional prefix (`feat:`, `fix:`, `docs:`, `chore:`).

## Security

See [SECURITY.md](SECURITY.md). Do not file vulnerabilities as public issues.

## License

By contributing you agree that your contribution is licensed under the MIT
License in [LICENSE](LICENSE).
