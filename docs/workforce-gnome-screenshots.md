# GNOME Wayland Screenshot Integration

Source adapter: \`apps/desktop/src-tauri/src/linux_screenshots.rs\`.

The desktop adapter does not bypass Wayland. It reads single frames from a
user-granted ScreenCast + PipeWire session maintained by the MIT-licensed
[wayland-screenshot](https://github.com/dtg01100/wayland-screenshot) CLI.

This external program is **not bundled or installed** by this branch.
Review/pin its version, install on a GNOME test machine, and in the graphical
user session run:

\`\`\`sh
wayland-screenshot doctor
wayland-screenshot cast enroll --monitor
wayland-screenshot cast start
\`\`\`

The agent tests readiness via \`wayland-screenshot cast status --json\` and
requests frames using \`wayland-screenshot snap -o PATH\`. It skips capture
when paused, idle, without first-run consent, without known focused app,
or while an excluded application is focused. It compresses PNG to WebP and
calls BiBo's existing \`insert_screenshot\` and multipart sync pipeline.

**Important limitations**

- ScreenCast permission requires an interactive GNOME portal grant.
- Captures are of the authorized monitor, **not the active window**.
- Background windows, notifications and other displayed private material are
  not redacted; the focused-app skip list cannot protect them.
- The helper CLI's exact command/JSON behavior and long-running session lifecycle
  have not been validated against the installed version.
- Unlock/reboot/revocation and multi-monitor behavior must be tested on GNOME Wayland.
- Do not distribute as unattended employee surveillance or imply capture is complete
  when the portal is unavailable.
