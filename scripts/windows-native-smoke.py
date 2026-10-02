#!/usr/bin/env python3
"""Opt-in installed CLI smoke. No production authentication, job or inference.

Uses Desktop's Rust supervisor; requires the accepted separately installed CLI.
Includes only a synthetic loopback rejected login. Ordinary CI uses fake binaries.
"""
import ctypes
from ctypes import wintypes as W
import json
import http.server
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import threading
import unittest

ROOT = Path(__file__).resolve().parents[1]
if os.name != "nt":
    raise SystemExit("This acceptance driver requires native Windows AMD64")
K = ctypes.WinDLL("kernel32", use_last_error=True)
K.OpenProcess.argtypes = [W.DWORD, W.BOOL, W.DWORD]
K.OpenProcess.restype = W.HANDLE
K.CloseHandle.argtypes = [W.HANDLE]
K.WaitForSingleObject.argtypes = [W.HANDLE, W.DWORD]
K.IsProcessInJob.argtypes = [W.HANDLE, W.HANDLE, ctypes.POINTER(W.BOOL)]
K.CreateToolhelp32Snapshot.argtypes = [W.DWORD, W.DWORD]
K.CreateToolhelp32Snapshot.restype = W.HANDLE


class ProcessEntry(ctypes.Structure):
    _fields_ = [("dwSize", W.DWORD), ("cntUsage", W.DWORD), ("pid", W.DWORD),
                ("heap", ctypes.c_size_t), ("module", W.DWORD), ("threads", W.DWORD),
                ("parent", W.DWORD), ("priority", W.LONG), ("flags", W.DWORD),
                ("name", W.WCHAR * 260)]


def descendants(pid):
    snapshot = K.CreateToolhelp32Snapshot(2, 0)
    if snapshot == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    entries = {}
    entry = ProcessEntry()
    entry.dwSize = ctypes.sizeof(entry)
    K.Process32FirstW.argtypes = [W.HANDLE, ctypes.POINTER(ProcessEntry)]
    K.Process32NextW.argtypes = [W.HANDLE, ctypes.POINTER(ProcessEntry)]
    try:
        more = K.Process32FirstW(snapshot, ctypes.byref(entry))
        while more:
            entries[entry.pid] = entry.parent
            more = K.Process32NextW(snapshot, ctypes.byref(entry))
    finally:
        K.CloseHandle(snapshot)
    found = {pid}
    while True:
        children = {p for p, parent in entries.items() if parent in found}
        if children <= found:
            return found - {pid}
        found |= children


class Watch:
    def __init__(self, pid):
        self.handle = K.OpenProcess(0x100000 | 0x1000, False, pid)
        if not self.handle:
            raise ctypes.WinError(ctypes.get_last_error())
        member = W.BOOL()
        if not K.IsProcessInJob(self.handle, None, ctypes.byref(member)) or not member.value:
            self.close()
            raise AssertionError("Desktop-owned CLI process is outside a Job Object")

    def assert_exit(self):
        if K.WaitForSingleObject(self.handle, 5000) != 0:
            raise AssertionError("Desktop-owned CLI process survived cleanup")

    def close(self):
        if self.handle:
            K.CloseHandle(self.handle)
            self.handle = None


class InstalledCliSmoke(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cli = Path(os.environ["LOCALAPPDATA"]) / "Programs/JV CLI/jvcli.exe"
        if not cls.cli.is_file():
            raise AssertionError("Accepted per-user jvcli.exe must be installed first")
        node = ROOT / ".tools/node/node.exe"
        if not node.is_file():
            node = Path("node")
        subprocess.run([str(node), "scripts/cargo.mjs", "build", "-p", "jv-desktop-process",
                        "--features", "test-fixture", "--bin", "bridge-owner", "--locked"],
                       cwd=ROOT, check=True)
        cls.owner = ROOT / "target/debug/bridge-owner.exe"

    def run_smoke(self, abrupt, auth_failure=False):
        (ROOT / ".cache/tmp").mkdir(parents=True, exist_ok=True)
        root = Path(tempfile.mkdtemp(prefix="installed-desktop-", dir=ROOT / ".cache/tmp"))
        workspace = root / "work space \u0e44\u0e17\u0e22"
        workspace.mkdir()
        # Deliberately missing child and parent: CLI owns private-state creation.
        state = root / "AppLocalData/com.openjvspace.desktop/jvcli-state"
        terminal = Path(os.environ["LOCALAPPDATA"]) / "JV/jv-cli"
        self.assertNotEqual(state, terminal)
        env = {k: os.environ[k] for k in ("SYSTEMROOT", "WINDIR", "USERPROFILE", "LOCALAPPDATA", "TEMP", "TMP") if k in os.environ}
        env["PATH"] = str(Path(os.environ["SYSTEMROOT"]) / "System32")
        for key in ("JV_API_PASSWORD", "JV_API_TOKEN", "OPENAI_API_KEY", "PARENT_SECRET", "PYTHONPATH"):
            env[key] = "SYNTHETIC_PARENT_SENTINEL"
        # Python is only this harness's unrelated sentinel, not a CLI dependency.
        sentinel = subprocess.Popen([sys.executable, "-B", "-c", "import time; time.sleep(300)"],
                                    stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        requests = []
        server = None
        server_thread = None
        args = [str(self.owner), str(self.cli), str(workspace), str(state)]
        if auth_failure:
            class RejectSyntheticAuth(http.server.BaseHTTPRequestHandler):
                def do_POST(self):
                    size = int(self.headers.get("Content-Length", "0"))
                    if size > 32768:
                        self.send_error(413)
                        return
                    self.rfile.read(size)  # Discard; no request/credential recording.
                    requests.append(True)
                    body = b'{"error":"synthetic authentication rejected"}'
                    self.send_response(401)
                    self.send_header("Content-Type", "application/json")
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)

                def log_message(self, *args):
                    pass
            server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), RejectSyntheticAuth)
            server_thread = threading.Thread(target=server.serve_forever)
            server_thread.start()
            args.append(f"http://127.0.0.1:{server.server_port}")
        owner = subprocess.Popen(args,
                                 cwd=root, env=env, stdin=subprocess.PIPE,
                                 stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        watches = []
        try:
            deadline = time.monotonic() + 15
            while time.monotonic() < deadline:
                if owner.poll() is not None:
                    self.fail("Rust supervisor rejected real CLI readiness")
                try:
                    bridge = json.loads((workspace / "owner.json").read_text())["bridge"]
                    break
                except (FileNotFoundError, json.JSONDecodeError):
                    time.sleep(.02)
            else:
                self.fail("Rust supervisor readiness deadline exceeded")
            # Readiness is unauthenticated and deliberately does not create state.
            # The fixed state environment is verified by the native peer tests;
            # this real smoke must not create an inherited-ACL child itself.
            if auth_failure:
                self.assertTrue(requests, "Synthetic login never reached the loopback fixture")
                self.assertTrue(state.is_dir(), "CLI did not create its Desktop-specific private state")
                ps = Path(os.environ["SYSTEMROOT"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
                escaped = str(state).replace("'", "''")
                script = f"$a=Get-Acl -LiteralPath '{escaped}'; if (-not $a.AreAccessRulesProtected) {{ exit 1 }}"
                subprocess.run([str(ps), "-NoProfile", "-NonInteractive", "-Command", script], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                for path in root.rglob("*"):
                    if path.is_file():
                        self.assertNotIn(b"TEST_PASSWORD_DO_NOT_LEAK", path.read_bytes())
            else:
                self.assertFalse(state.exists(), "Unauthenticated startup unexpectedly created state")
            self.assertEqual(descendants(owner.pid) >= {bridge}, True)
            ids = descendants(owner.pid)
            self.assertGreaterEqual(len(ids), 2, "Packaged CLI launcher/runtime tree missing")
            watches = [Watch(pid) for pid in ids]
            if abrupt:
                owner.kill()  # Only our supervisor parent; OS job close cleans the tree.
            else:
                owner.stdin.close()  # Rust supervisor sends bridge.shutdown, then waits.
            self.assertEqual(owner.wait(timeout=15), 1 if abrupt else 0)
            for watch in watches:
                watch.assert_exit()
            self.assertIsNone(sentinel.poll(), "Unrelated process was terminated")
            self.assertEqual(state.exists(), auth_failure)
        finally:
            if owner.stdin and not owner.stdin.closed:
                owner.stdin.close()
            if owner.poll() is None:
                owner.kill()
            owner.wait(timeout=10)
            for watch in watches:
                watch.close()
            sentinel.kill()
            sentinel.wait(timeout=10)
            if server:
                server.shutdown()
                server.server_close()
                server_thread.join(timeout=5)
                self.assertFalse(server_thread.is_alive())

    def test_real_cli_version_ready_workspace_state_stderr_and_shutdown(self):
        self.run_smoke(False)

    def test_real_cli_nested_job_parent_death_and_unrelated_survival(self):
        self.run_smoke(True)

    def test_synthetic_rejected_auth_creates_private_desktop_state_without_secret_persistence(self):
        self.run_smoke(False, auth_failure=True)


if __name__ == "__main__":
    unittest.main(verbosity=2)
