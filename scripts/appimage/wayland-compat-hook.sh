#! /usr/bin/env bash
#
# wayland-compat-hook.sh — AppRun hook that makes the AppImage usable on a
# modern Wayland desktop. Installed into the AppImage's apprun-hooks/ by
# scripts/patch-appimage-wayland.sh; never used at development time.
#
# WHY THIS EXISTS
#   linuxdeploy follows libgtk-3's dependencies and copies libwayland-client,
#   -cursor, -egl and -server into the AppImage, and the app binary carries
#   RUNPATH=$ORIGIN/../lib, so those bundled copies win over the host's. libEGL
#   is NOT bundled, so the host's Mesa is what actually runs — and a current
#   Mesa driving a libwayland from the (deliberately old) build image fails:
#
#     Could not create default EGL display: EGL_BAD_PARAMETER. Aborting...
#
#   WebKitWebProcess then aborts and the user gets an empty white window.
#
# WHAT IT DOES
#   Under a Wayland session, preload the host's libwayland so the loader
#   resolves those sonames to the system copies (matching the host's Mesa)
#   instead of the bundled ones. The bundled libraries stay in the AppImage as
#   a fallback: if the host has no libwayland at all, this hook does nothing and
#   the AppImage keeps working exactly as before.
#
#   ARM_WAYLAND_LIB_DIRS overrides the search path (colon-separated) — used by
#   the tests, and an escape hatch for unusual library layouts.

if [ -n "${WAYLAND_DISPLAY:-}" ] || [ "${XDG_SESSION_TYPE:-}" = "wayland" ]; then
  arm_wayland_lib_dirs="${ARM_WAYLAND_LIB_DIRS:-/usr/lib/x86_64-linux-gnu:/usr/lib64:/usr/lib:/lib/x86_64-linux-gnu}"
  arm_wayland_preload=""

  for arm_wayland_lib in \
    libwayland-client.so.0 \
    libwayland-egl.so.1 \
    libwayland-cursor.so.0 \
    libwayland-server.so.0; do
    arm_wayland_saved_ifs="${IFS}"
    IFS=":"
    for arm_wayland_dir in $arm_wayland_lib_dirs; do
      if [ -e "$arm_wayland_dir/$arm_wayland_lib" ]; then
        arm_wayland_preload="${arm_wayland_preload:+$arm_wayland_preload:}$arm_wayland_dir/$arm_wayland_lib"
        break
      fi
    done
    IFS="${arm_wayland_saved_ifs}"
  done

  # Keep any LD_PRELOAD the user set; ours only takes precedence over it.
  if [ -n "$arm_wayland_preload" ]; then
    export LD_PRELOAD="${arm_wayland_preload}${LD_PRELOAD:+:$LD_PRELOAD}"
  fi

  unset arm_wayland_lib_dirs arm_wayland_preload arm_wayland_lib \
    arm_wayland_dir arm_wayland_saved_ifs
fi
