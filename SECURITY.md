# Security policy

## Supported versions

No stable release is available yet; this is an initial source snapshot, not a reviewed security-hardened distribution.

## Reporting vulnerabilities

After repository publication, use GitHub's private vulnerability reporting feature where enabled. Do not post exploit details publicly before coordinating remediation. Until then, report issues privately to the repository maintainer after the contact path is published. No security email has been invented in this template.

## Threat model

- Bandhop runs with the user's privileges, **never root**.
- The control socket is in a private (0700) user runtime directory.
- No remote control TCP/UDP port is exposed.
- External HTTP(S) radio streams are consumed by `mpv`; treat URLs as untrusted.
- Shell invocation is not used for station URLs or notification content.
- KDE global shortcuts are registered through the desktop's configuration UI.

Do not manually expose the control socket to other users, do not use `chmod 777`, and do not launch the daemon as root. Keep system and mpv updated.

## Known limitations

This build does not authenticate processes beyond filesystem permissions on its socket directory; processes running as the same user can issue local commands. Foreground shell termination may not handle every crash scenario. Network reconnection and stream health reporting are outside v0.1.
