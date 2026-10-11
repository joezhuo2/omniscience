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
   `src-tauri/tauri.conf.json` at `plugins.updater.pubkey`, replacing
   `REPLACE_WITH_TAURI_UPDATER_PUBLIC_KEY`, and commit it.
2. In the GitHub repository settings, open Secrets and variables > Actions and add:
   - `TAURI_SIGNING_PRIVATE_KEY`: the contents of `~/.tauri/omniscience.key`
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: the password you chose (empty if none)
3. Keep the private key safe and backed up. If it is lost, existing installs can no longer
   verify updates and users have to reinstall manually.

Until the public key is set, update checks still work, but installing an update fails signature
verification and the banner shows the error.

## Cutting a release

1. Bump the version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
   (all three must match).
2. Move the `[Unreleased]` entries in `CHANGELOG.md` under a new version heading.
3. Commit, then tag and push:

   ```bash
   git tag v0.1.0
   git push origin main --tags
   ```

4. When the workflow finishes, review the draft release and publish it. The updater endpoint
   (`releases/latest/download/latest.json`) only serves **published** releases, so installed
   copies see the update once you publish.

## Unsigned builds

The binaries are not code-signed (no Authenticode certificate or Apple notarization), so:

- **Windows:** SmartScreen shows "Windows protected your PC". Choose More info > Run anyway.
- **macOS:** Gatekeeper blocks the first launch. Right-click the app > Open, or allow it in
  System Settings > Privacy & Security. macOS support is experimental and untested.
