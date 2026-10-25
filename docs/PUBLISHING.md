# Publishing Bandhop to GitHub

**Release gate:** first build and test v0.1.2 on Linux, confirm F13/F14, station cycling, stop, and visible desktop notifications. Do not publish a tagged release before these checks pass.

Bandhop is published under `MIT OR Apache-2.0`. Ensure the author and contact information are accurate before publication. Cargo.toml and README are prepared for `husamettinarabaci/bandhop`. The release-PKGBUILD still uses a `@GITHUB_OWNER@` template placeholder and must be populated before an AUR submission.

## Create a public repository

GitHub's CLI can create the repository and push an existing Git checkout:

```sh
sudo pacman -S --needed github-cli git
cd ~/Downloads/bandhop-0.1.2
cargo test --locked
./scripts/smoke-test.sh
cargo build --release --locked
git init -b main
git add .
git commit -m "feat: initial Bandhop Rust CLI and desktop notifications"
gh auth login
gh repo create husamettinarabaci/bandhop --public --source=. --remote=origin --push
```

If a repository is already created, do **not** run `gh repo create` again. Instead configure the remote and use `git push -u origin main`. Review `git status`, `git diff --cached --stat`, and `git ls-files` before the initial public push so no personal configuration, tokens, or generated `target/` files are published.

## After the initial push

1. Confirm `https://github.com/husamettinarabaci/bandhop` is public and all documentation renders correctly.
2. Wait for the GitHub Actions CI workflow to pass.
3. Check that repository links and badges resolve correctly, and update any remaining publication-specific template placeholders.
4. Tag the verified commit (`git tag -a v0.1.2 -m "Bandhop v0.1.2"`; `git push origin v0.1.2`). The release workflow will attempt to publish archives and checksums.
5. Only after the GitHub release and checksum verification, populate `PKGBUILD.release.in` placeholders and use the AUR submission process. Inclusion in CachyOS's official repositories is not automatic.

These commands are **instructions**, not evidence that publication already happened.
