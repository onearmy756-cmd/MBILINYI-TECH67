# Building the phone and desktop apps

The platform is served from one place, so the apps are thin: each one opens
`https://mbilinyitech.co.tz` and adds what a browser cannot — an installable
icon on the home screen, a real window on the desktop, and access to the
device's camera, microphone and notifications.

This file says exactly how to build each target, and states plainly which parts
were verified on the machine the change was written on and which need a machine
with the platform's own toolchain.

## What was verified, and what was not

| Target | Built here? | Evidence |
|--------|-------------|----------|
| Linux `.deb` | **Yes** | `npx tauri build --bundles deb` produced `Mbilinyi Tech_1.0.0_amd64.deb` (~1.4 MB) containing the binary, the `.desktop` entry and the brand icons |
| Linux `.AppImage` | **No** | Bundling needs FUSE, which the build container does not have. The `.deb` proves the crate and the assets are sound; only the packaging step is untested |
| Android `.apk` / `.aab` | **No** | No Java or Android SDK in the container. `cap add android` and `cap sync` both complete, and the generated project was inspected, but Gradle never ran |
| iOS `.ipa` | **No** | Needs macOS and Xcode |
| Windows `.msi` / `.exe` | **No** | Needs a Windows host, or a Linux host with `cargo-xwin` |

An untested target is not a broken one: the parts that are shared — the crate,
the config, the icons, the plugin wiring — are the parts that were exercised.
The remaining step is a machine with the platform's toolchain, not a code change.

## The desktop app (Tauri 2)

```bash
cd desktop
npm install
npm run build:linux      # .deb and .AppImage
npm run build:windows    # .msi and .exe      — on Windows
npm run build:mac        # .dmg and .app      — on macOS
```

One prerequisite matters on Linux: the window uses webkit2gtk. Debian 13 carries
the 4.1 generation, so install the 4.1 development package, not 4.0.

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev \
                        libayatana-appindicator3-dev patchelf
```

The app is a window onto the live platform. It asks for no filesystem access, no
shell and no device permissions — `desktop/src-tauri/capabilities/default.json`
grants `core:default` and nothing else. Its content policy is the same policy
the server sends, because a narrower rule would break the page the window exists
to show.

## The phone app (Capacitor)

```bash
cd mobile
npm install
npx cap add android          # first time only; android/ is gitignored
npx cap sync                 # after any config change
npm run build:android        # needs the Android SDK and a JDK
```

iOS, on a Mac:

```bash
npx cap add ios
npm run build:ios
```

The phone app also points at the live platform, so an installed copy always runs
the current build and no stale copy can be left on a device.

### Icons and splash screens

Both apps carry the company mark, generated from the same source as the website:
the "M" on the Obsidian Ember gradient (`#130A07` plate, `#FF6B2C` mark).

```bash
cd mobile
npx capacitor-assets generate --android   # or --ios
```

The generator reads `mobile/resources/icon.png`, `icon-background.png`,
`splash.png` and `splash-dark.png`. Two details are easy to get wrong and both
were:

* **The foreground must be transparent** outside the mark. An opaque foreground
  covers the plate behind it, so the launcher shows a square of colour instead
  of the logo. `mobile/resources/icon.png` is transparent; check it with any
  image viewer that shows alpha.
* **The plate must be the brand dark**, not the white the template ships with.
  `mobile/android/app/src/main/res/values/ic_launcher_background.xml` holds the
  colour a launcher falls back to.

The desktop icons live in `desktop/src-tauri/icons/` as a multi-image `.ico`
(Windows), an `.icns` (macOS) and a PNG set (Linux).

## Checking the platform itself

```bash
cd rust/mts-core
cargo test --release        # 97 tests
cargo fmt --all -- --check
cargo clippy --all-targets
```

## Live calls

A call carries audio and video straight between the participants, so the server
never sees the conversation — it relays only the handshake that introduces two
people to each other.

* Joining a room **requires a signed-in session**, and the name shown in a call
  is taken from the account, not from the URL.
* A public STUN server gets two browsers behind home routers talking. It cannot
  get through the symmetric NAT a mobile carrier uses, so calls on a phone
  network need a **TURN** server.
* The CEO adds one in *Admin → Settings → `call_ice_servers`*, as a JSON array:

  ```json
  [{ "urls": "turn:turn.example.com:3478", "username": "user", "credential": "secret" }]
  ```

  It reaches the browser as `window.__ICE__` on the next page load. Until a TURN
  server is set, a call on a mobile network may fail to connect and the caller
  is told so in plain words.
