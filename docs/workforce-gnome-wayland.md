# Workforce Intelligence — GNOME Wayland agent branch

This is a development integration into BiBoTracking, **not a finished Linux monitoring release**.

## Current implementation in this commit

- Linux OS selector in the Rust platform abstraction, preserving existing Windows and macOS backends.
- GNOME Shell extension exposing only the frontmost application ID via session D-Bus.
- GNOME Mutter IdleMonitor adapter; unknown/unavailable idle signals are treated as unknown, never active.
- Normal BiBo local app sampling, SQLite activity storage, backend sync, browser bridge, and React dashboard remain in place.
- Linux keyboard hooks and existing \`xcap\` screenshots are intentionally disabled: neither should silently run without a vetted Wayland integration.

## GNOME focus extension evaluation (GNOME 46–48 only; unverified)

Install the extension for the *logged-in* user:

\`\`\`bash
install -d ~/.local/share/gnome-shell/extensions/focus@workforce-intelligence.local
cp integrations/gnome-focus/{extension.js,metadata.json} \
  ~/.local/share/gnome-shell/extensions/focus@workforce-intelligence.local/
gnome-extensions enable focus@workforce-intelligence.local
gdbus call --session \
  --dest org.workforceintelligence.Focus \
  --object-path /org/workforceintelligence/Focus \
  --method org.workforceintelligence.Focus1.GetFocus
\`\`\`

May require a logout/login before GNOME detects an extension. No window titles are exported.
The extension API, GNOME compatibility and Rust build must be tested on the target distribution.

## Remaining before production

1. Implement GNOME ScreenCast portal and PipeWire screenshot capture, with explicit user permission.
2. Wire authorized keyboard/mouse **count-only** service into SQLite and sync without key identities or coordinates.
3. Add permission state, session-lock behavior and collection coverage indicators.
4. Enforce browser **domain-only** data at the collection boundary.
5. Implement server-side productivity scoring and department history sourced from Selamnew.
6. Compile with Rust/Tauri on GNOME Wayland and perform cross-platform regression tests.
7. Replace upstream identifier, updater URL and public signing key before distribution.

Do not deploy to employees until consent, data retention, authorization, and security reviews pass.
