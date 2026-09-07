# Logging

| Item | Policy |
| --- | --- |
| Location | `$XDG_CACHE_HOME/skwd-wall-v2`, falling back to `~/.cache/skwd-wall-v2` |
| Active file | Up to 4 MiB, checked during writes |
| Retention | Active file plus `.1`, `.2`, and `.3`; oldest backup removed on rotation |
| Permissions | Log and lock files use mode 0600 |
| Concurrent writers | A per-log file lock covers size checks, rotation, and appends |
| File failure | Application work continues; scanner output is drained even if its log cannot be written |
| Terminal and service output | Stderr remains available; systemd journal retention belongs to journald |

Wall, Deck, and Paper use the same rotating writer. Deck owns `skwd-log`; standalone Paper owns `paper-log`. Their writer implementation and regression tests must match; Deck's parity gate checks both. Paper has no dependency on Deck.

The policy covers application logs, scanner stdout/stderr capture, selector timing logs, and daemon vitals. Explicit profiling captures such as Chrome traces are output artifacts and retain their requested format; they are not rotated as text logs. Shell redirections and system journals have their own retention settings.

A scanner receives a pipe for diagnostics. The daemon drains it into the rotating writer, so the helper's file-size resource limit cannot kill it for writing to an inherited full log. Scanner resource limits remain enabled.

An oversized file from an older version is moved into the backup sequence on the next write. It ages out after three subsequent rotations; it is not truncated during migration. After those old files age out, each log family uses at most 16 MiB, plus an empty lock file. There is no background timer or logging work while idle.
