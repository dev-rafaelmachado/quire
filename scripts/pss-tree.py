#!/usr/bin/env python3
"""Memory of a running Quire (DEB-07): PSS of a process and all its descendants, Linux only.

Usage: scripts/pss-tree.py <pid of quire>
Prints "total_MB webkit_MB processes", summing /proc/<pid>/smaps_rollup over the process tree
(the app and its WebKitWebProcess / WebKitNetworkProcess children), the way the v0.1 spike
(T2) and v0.2 T6 measured it.
"""
import os
import sys


def children(pid: int) -> list[int]:
    out: list[int] = []
    for task in os.listdir(f"/proc/{pid}/task"):
        try:
            with open(f"/proc/{pid}/task/{task}/children") as f:
                out += [int(x) for x in f.read().split()]
        except OSError:
            pass
    return out


def pss_kb(pid: int) -> int:
    try:
        with open(f"/proc/{pid}/smaps_rollup") as f:
            for line in f:
                if line.startswith("Pss:"):
                    return int(line.split()[1])
    except OSError:
        pass
    return 0


def cmdline(pid: int) -> str:
    try:
        with open(f"/proc/{pid}/cmdline") as f:
            return f.read()
    except OSError:
        return ""


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    todo, seen = [int(sys.argv[1])], []
    while todo:
        pid = todo.pop()
        seen.append(pid)
        todo += children(pid)
    total = sum(pss_kb(p) for p in seen)
    webkit = sum(pss_kb(p) for p in seen if "WebKitWebProcess" in cmdline(p))
    print(f"{total / 1024:.0f} {webkit / 1024:.0f} {len(seen)}")


if __name__ == "__main__":
    main()
