#!/usr/bin/env bash
# Entrypoint for Dockerfile.builder. Copies any bind-mounted
# /host-ssh files into /root/.ssh with root ownership + correct
# perms, then exec's the user's command.
#
# Background: SSH refuses configs not owned by the current user
# ("Bad owner or permissions on /root/.ssh/config"). A bind-mount
# of the runner's ~/.ssh -> /root/.ssh preserves the runner uid
# (1001), but the container runs as root (uid 0), so SSH refuses.
# Mounting to /host-ssh:ro + copying + chown'ing here is the
# minimal fix.
#
# Silently skipped when /host-ssh is not mounted (e.g. local
# `docker run` for ad-hoc inspection).

set -e
if [ -d /host-ssh ]; then
    mkdir -p /root/.ssh
    # The cp may fail on a stale broken symlink; allow it.
    cp -L /host-ssh/* /root/.ssh/ 2>/dev/null || true
    chown -R root:root /root/.ssh
    chmod 700 /root/.ssh
    # Glob may be empty if /host-ssh was empty — don't fail.
    if compgen -G '/root/.ssh/*' > /dev/null; then
        chmod 600 /root/.ssh/*
    fi
fi
exec "$@"
