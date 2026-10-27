# Releasing

Releases are built by `.github/workflows/release.yml` when a `v*` tag is pushed. The workflow:

- builds the Windows installers (`.msi`, NSIS `.exe`) and a universal macOS `.dmg`;
- signs the updater artifacts;
- creates a **draft** GitHub Release with `latest.json` attached.

## One-time setup: updater signing key

The in-app updater only installs updates signed with your key. Generate a keypair once, on your
own machine:

```bash
npx tauri signer generate -w ~/.tauri/omniscience.key
```

Then:

1. Paste the **public** key (the contents of `~/.tauri/omniscience.key.pub`) into
   `src-tauri/tauri.conf.json` at `plugins.updater.pubkey`, and commit it. This is done: the
   committed key has minisign ID `37B65D49D1BAA7DB`.
2. In the GitHub repository settings, open Secrets and variables > Actions and add:
   - `TAURI_SIGNING_PRIVATE_KEY`: the contents of `~/.tauri/omniscience.key`
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: the password you chose (empty if none)
3. Keep the private key safe and backed up. If it is lost, existing installs can no longer
   verify updates and users have to reinstall manually.

Every installed copy trusts the public key it shipped with. Replacing the keypair later means
installs of earlier versions can no longer update themselves.

## Cutting a release

1. Bump the version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
   (all three must match), and synchronize the root app entries in `package-lock.json` and
   `src-tauri/Cargo.lock`. `npm run version:check` verifies the manifests; CI runs it on every push, and
   the release workflow also fails if the tag does not match.
2. Move the `[Unreleased]` entries in `CHANGELOG.md` under a new version heading.
3. Commit in small logical changes, push a release branch, and open a PR. After CI passes,
   merge the PR and tag the merge commit on `main` (replace `vX.Y.Z` with the release version):

   ```bash
   git switch main
   git pull --ff-only
   npm run version:check -- vX.Y.Z
   git tag -a vX.Y.Z -m "Omniscience vX.Y.Z"
   git push origin vX.Y.Z
   ```

4. When the workflow finishes, review the draft release and publish it. The updater endpoint
   (`releases/latest/download/latest.json`) only serves **published** releases, so installed
   copies see the update once you publish.

## Unsigned builds

The binaries are not code-signed (no Authenticode certificate or Apple notarization), so:

- **Windows:** SmartScreen shows "Windows protected your PC". Choose More info > Run anyway.
- **macOS:** Gatekeeper blocks the first launch. Right-click the app > Open, or allow it in
  System Settings > Privacy & Security. macOS support is experimental and untested.
