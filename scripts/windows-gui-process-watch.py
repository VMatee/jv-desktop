#!/usr/bin/env python3
"""Process-handle assertions for native manual-action GUI acceptance.

No UI automation, real account, network client or process-name termination.
Only the harness's fixture PIDs are watched; the unrelated sentinel is owned
and stopped by this helper after its survival has been checked.
"""
import argparse
import ctypes
from ctypes import wintypes as W
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
if os.name != "nt":
    raise SystemExit("Native Windows required")
spec = importlib.util.spec_from_file_location("native_smoke", ROOT / "scripts/windows-native-smoke.py")
smoke = importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)
K = smoke.K
K.Process32FirstW.argtypes = [W.HANDLE, ctypes.POINTER(smoke.ProcessEntry)]
K.Process32NextW.argtypes = [W.HANDLE, ctypes.POINTER(smoke.ProcessEntry)]
K.GetProcessTimes.argtypes = [W.HANDLE] + [ctypes.POINTER(W.FILETIME)] * 4
K.GetProcessTimes.restype = W.BOOL


def parents():
    snapshot = K.CreateToolhelp32Snapshot(2, 0)
    if snapshot == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    result = {}
    entry = smoke.ProcessEntry()
    entry.dwSize = ctypes.sizeof(entry)
    try:
        more = K.Process32FirstW(snapshot, ctypes.byref(entry))
        while more:
            result[entry.pid] = entry.parent
            more = K.Process32NextW(snapshot, ctypes.byref(entry))
    finally:
        K.CloseHandle(snapshot)
    return result


class Watch:
    def __init__(self, pid, require_job=False):
        self.pid = pid
        self.handle = K.OpenProcess(0x100000 | 0x1000, False, pid)
        if not self.handle:
            raise ctypes.WinError(ctypes.get_last_error())
        if require_job:
            member = W.BOOL()
            if not K.IsProcessInJob(self.handle, None, ctypes.byref(member)) or not member.value:
                self.close()
                raise AssertionError("Fixture process is outside a Job Object")

    def alive(self):
        return K.WaitForSingleObject(self.handle, 0) == 258

    def exited(self, timeout=0):
        return K.WaitForSingleObject(self.handle, timeout) == 0

    def times(self):
        values = [W.FILETIME() for _ in range(4)]
        if not K.GetProcessTimes(self.handle, *(ctypes.byref(v) for v in values)):
            raise ctypes.WinError(ctypes.get_last_error())
        return tuple((v.dwHighDateTime << 32) | v.dwLowDateTime for v in values[:2])

    def close(self):
        if self.handle:
            K.CloseHandle(self.handle)
            self.handle = None


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--sentinel-dir", type=Path, required=True)
    args = parser.parse_args()
    cache = (ROOT / ".cache/manual-gui").resolve()
    for path in (args.fixture, args.sentinel_dir):
        if not path.resolve().is_relative_to(cache):
            raise AssertionError("GUI fixtures must stay inside the disposable acceptance directory")
    env = {key: os.environ[key] for key in ("SYSTEMROOT", "WINDIR", "USERPROFILE", "LOCALAPPDATA", "TEMP", "TMP") if key in os.environ}
    env["PATH"] = str(Path(os.environ["SYSTEMROOT"]) / "System32")
    sentinel = subprocess.Popen([str(args.fixture), "--fixture-grandchild"],
                                cwd=args.sentinel_dir, env=env, stdin=subprocess.DEVNULL,
                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    watches = {}
    groups = {}
    desktop_pid = None

    def watch(label, pid, job=False):
        if label not in watches:
            watches[label] = Watch(pid, job)
        return watches[label]

    def request(command):
        nonlocal desktop_pid
        kind = command["type"]
        if kind == "desktop":
            desktop_pid = int(command["pid"])
            watch("desktop", desktop_pid)
            watch("sentinel", sentinel.pid)
            return {"desktop": desktop_pid, "sentinel": sentinel.pid}
        if kind == "tree":
            workspace = Path(command["workspace"]).resolve()
            if not workspace.is_relative_to(cache):
                raise AssertionError("Unexpected fixture workspace")
            child = json.loads((workspace / "child.json").read_text())["pid"]
            grandchild = json.loads((workspace / "grandchild.json").read_text())["pid"]
            graph = parents()
            bridge = graph[child]
            assert graph[bridge] == desktop_pid, "Bridge is not the Desktop's child"
            assert graph[grandchild] == child, "Grandchild is not owned by the fixture child"
            group = command["group"]
            groups[group] = [watch(f"{group}-{name}", pid, True) for name, pid in
                             (("bridge", bridge), ("child", child), ("grandchild", grandchild))]
            assert all(w.alive() for w in groups[group]), "Fixture tree exited early"
            return {"bridge": bridge, "child": child, "grandchild": grandchild, "job": True}
        if kind == "unchanged":
            assert all(w.alive() for w in groups[command["group"]]), "Dialog cancellation terminated the bridge"
            return {"bridge_unchanged": True}
        if kind == "changed":
            old = groups[command["old"]]
            new = groups[command["new"]]
            assert all(w.exited(5000) for w in old), "Previous workspace tree survived replacement"
            new_created = new[0].times()[0]
            assert 0 < old[0].times()[1] <= new_created, "New bridge began before the previous bridge exited"
            return {"previous_bridge_exited_before_new_bridge": True, "previous_owned_tree_exited": True}
        if kind == "before-close":
            # Snapshot only the launched app's descendants; retain handles to
            # prevent PID reuse from masquerading as a cleanup success.
            ids = smoke.descendants(desktop_pid)
            assert len(ids) <= 128, "Unexpectedly large Desktop process tree"
            for pid in ids:
                label = f"desktop-descendant-{pid}"
                try:
                    watch(label, pid)
                except OSError:
                    if pid in smoke.descendants(desktop_pid):
                        raise
            return {"owned_descendants_watched": len(ids)}
        if kind == "closed":
            assert watches["desktop"].exited(20000), "Desktop did not exit after the native close action"
            for label, handle in watches.items():
                if label != "sentinel":
                    assert handle.exited(5000), f"Owned process survived native close: {label}"
            assert not smoke.descendants(desktop_pid), "A hidden owned Desktop descendant remains"
            assert watches["sentinel"].alive() and sentinel.poll() is None, "Unrelated sentinel was terminated"
            return {"desktop_exited": True, "bridge_exited": True,
                    "child_exited": True, "grandchild_exited": True,
                    "no_hidden_owned_process": True, "unrelated_survived": True}
        raise AssertionError("Unknown watcher request")

    try:
        while line := sys.stdin.buffer.readline(8193):
            if len(line) > 8192:
                raise AssertionError("Watcher command exceeds its limit")
            try:
                answer = {"ok": True, "result": request(json.loads(line))}
            except Exception as error:
                answer = {"ok": False, "error": str(error)}
            print(json.dumps(answer), flush=True)
    finally:
        if sentinel.poll() is None:
            sentinel.terminate()
        sentinel.wait(timeout=5)
        for handle in watches.values():
            handle.close()


if __name__ == "__main__":
    main()
