# Updater

Turning on in-app updates: generate a signing key pair, give the public half to
the app and the private half to CI, and point the app at your releases.

The wiring is already here — the plugin is registered, the release workflow
signs and publishes a `latest.json`, and
[`src-tauri/src/updater.rs`](../apps/desktop/src-tauri/src/updater.rs) checks
once at startup. It all stands down until `pubkey` is set, so the template
builds and releases fine before you do any of this.

## Generate the key pair

Location: your terminal, at the repository root.

1. Run `pnpm --filter @app/desktop exec tauri signer generate -w ~/.tauri/myapp.key`.
2. Enter a password when prompted, or leave it empty.
3. Keep the printed **public key** for the next section.
4. Back up `~/.tauri/myapp.key` somewhere you will still have it in a year.

**Losing the private key ends updates for every installed copy.** A new key
means a new `pubkey`, and an app already in the wild only trusts the one it
shipped with — those installs have to be replaced by hand.

Never commit either half of the key.

## Point the app at your releases

Location: [`apps/desktop/src-tauri/tauri.conf.json`](../apps/desktop/src-tauri/tauri.conf.json)

1. Set `plugins.updater.pubkey` to the public key from the previous section.
2. Replace `OWNER/REPO` in `plugins.updater.endpoints` with your repository.
3. Leave `bundle.createUpdaterArtifacts` set to `true`.

The endpoint uses `releases/latest/download`, so it always resolves to the
newest published release and never needs editing again.

## Give the private key to CI

Location: **GitHub → your repository → Settings → Secrets and variables →
Actions → Repository secrets** (`https://github.com/<owner>/<repo>/settings/secrets/actions`)

1. Add a secret named `TAURI_SIGNING_PRIVATE_KEY` holding the **contents** of
   `~/.tauri/myapp.key`, not its path.
2. Add a secret named `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` holding the password
   you chose. Create it holding an empty value if you chose none.

| Secret                               | Holds                           |
| ------------------------------------ | ------------------------------- |
| `TAURI_SIGNING_PRIVATE_KEY`          | The contents of the `.key` file |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Its password, or empty          |

## Set the product name in the manifest step

Location: [`.github/workflows/deploy.yml`](../.github/workflows/deploy.yml)

1. Set `--product-name` on the **Build the updater manifest** step to
   `productName` from `tauri.conf.json`.

It only names the two macOS tarballs, which the bundler gives identical names
and one release cannot hold two of.

## Verify

1. Push a `feat:` or `fix:` commit to `main` and let the release workflow run.
2. Open the finished GitHub release. It should carry `latest.json`, and a `.sig`
   beside every installer.
3. Open `latest.json`. It should list four platforms — `darwin-aarch64`,
   `darwin-x86_64`, `linux-x86_64`, `windows-x86_64` — each with a non-empty
   `signature`.
4. Install the previous release, then launch it. It should offer the new
   version, install it, and restart.

## Gotchas

- A release built before the secrets existed has no `.sig` files, so the
  **Build the updater manifest** step fails the deploy. That is deliberate — a
  manifest the updater would refuse is worse than none. Re-run the release once
  the secrets are set.
- `tauri.conf.json` holds the public key only. If a private key ever lands in
  the repo, rotate it.
- The startup check is skipped in `pnpm dev`: a development build cannot
  replace itself.
- Linux updates apply to the AppImage only. `.deb` and `.rpm` installs are
  updated by their package manager, not by the app.
- macOS and Windows still show an unidentified-developer warning on install.
  Updater signing is not OS code signing; see
  [the Tauri signing guide](https://v2.tauri.app/distribute/sign/) for that.
- Bumping `pubkey` in a release does not retro-fix older installs, which
  verify against the key they shipped with.

Background on the manifest and what the plugin does with it:
[@pixpilot/tauri-updater-manifest](https://github.com/ccpu/tauri-tools/tree/main/packages/updater-manifest)
and the [Tauri updater plugin](https://v2.tauri.app/plugin/updater/).
