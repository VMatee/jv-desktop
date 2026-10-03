#!/usr/bin/env python3
"""Run only the installed WebView smoke at normal Windows privileges.

Hosted runners are elevated. WebView2 150+ deliberately ignores environment CDP
flags for elevated hosts. Reduce this test child's token; never change accounts,
policies, registry security settings or the shipped application's configuration.
"""
import ctypes
from ctypes import wintypes as W
import os
import msvcrt
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
if os.name != 'nt':
    raise SystemExit('Native Windows required')
K = ctypes.WinDLL('kernel32', use_last_error=True)
A = ctypes.WinDLL('advapi32', use_last_error=True)


class SidAttributes(ctypes.Structure):
    _fields_ = [('sid', W.LPVOID), ('attributes', W.DWORD)]


class Startup(ctypes.Structure):
    _fields_ = [('cb', W.DWORD), ('reserved', W.LPWSTR), ('desktop', W.LPWSTR),
                ('title', W.LPWSTR), ('x', W.DWORD), ('y', W.DWORD),
                ('xsize', W.DWORD), ('ysize', W.DWORD), ('xchars', W.DWORD),
                ('ychars', W.DWORD), ('fill', W.DWORD), ('flags', W.DWORD),
                ('show', W.WORD), ('reserved_size', W.WORD), ('reserved2', W.LPVOID),
                ('stdin', W.HANDLE), ('stdout', W.HANDLE), ('stderr', W.HANDLE)]


class Process(ctypes.Structure):
    _fields_ = [('process', W.HANDLE), ('thread', W.HANDLE), ('pid', W.DWORD), ('tid', W.DWORD)]


class JobBasic(ctypes.Structure):
    _fields_ = [('process_time', ctypes.c_int64), ('job_time', ctypes.c_int64),
                ('flags', W.DWORD), ('min_working', ctypes.c_size_t),
                ('max_working', ctypes.c_size_t), ('active_limit', W.DWORD),
                ('affinity', ctypes.c_size_t), ('priority', W.DWORD), ('scheduling', W.DWORD)]


class IoCounters(ctypes.Structure):
    _fields_ = [(name, ctypes.c_uint64) for name in ('read', 'write', 'other', 'read_bytes', 'write_bytes', 'other_bytes')]


class JobLimits(ctypes.Structure):
    _fields_ = [('basic', JobBasic), ('io', IoCounters), ('process_memory', ctypes.c_size_t),
                ('job_memory', ctypes.c_size_t), ('peak_process', ctypes.c_size_t), ('peak_job', ctypes.c_size_t)]


K.GetCurrentProcess.restype = W.HANDLE
K.CloseHandle.argtypes = [W.HANDLE]
K.LocalFree.argtypes = [W.LPVOID]
K.GetStdHandle.argtypes = [W.DWORD]
K.GetStdHandle.restype = W.HANDLE
K.DuplicateHandle.argtypes = [W.HANDLE, W.HANDLE, W.HANDLE, ctypes.POINTER(W.HANDLE), W.DWORD, W.BOOL, W.DWORD]
K.CreateJobObjectW.argtypes = [W.LPVOID, W.LPCWSTR]
K.CreateJobObjectW.restype = W.HANDLE
K.SetInformationJobObject.argtypes = [W.HANDLE, ctypes.c_int, W.LPVOID, W.DWORD]
K.AssignProcessToJobObject.argtypes = [W.HANDLE, W.HANDLE]
K.ResumeThread.argtypes = [W.HANDLE]
K.ResumeThread.restype = W.DWORD
K.WaitForSingleObject.argtypes = [W.HANDLE, W.DWORD]
K.GetExitCodeProcess.argtypes = [W.HANDLE, ctypes.POINTER(W.DWORD)]
K.TerminateProcess.argtypes = [W.HANDLE, W.UINT]
A.OpenProcessToken.argtypes = [W.HANDLE, W.DWORD, ctypes.POINTER(W.HANDLE)]
A.GetTokenInformation.argtypes = [W.HANDLE, ctypes.c_int, W.LPVOID, W.DWORD, ctypes.POINTER(W.DWORD)]
A.CreateRestrictedToken.argtypes = [W.HANDLE, W.DWORD, W.DWORD, W.LPVOID, W.DWORD, W.LPVOID, W.DWORD, W.LPVOID, ctypes.POINTER(W.HANDLE)]
A.ConvertStringSidToSidW.argtypes = [W.LPCWSTR, ctypes.POINTER(W.LPVOID)]
A.SetTokenInformation.argtypes = [W.HANDLE, ctypes.c_int, W.LPVOID, W.DWORD]
A.GetLengthSid.argtypes = [W.LPVOID]
A.GetSidSubAuthorityCount.argtypes = [W.LPVOID]
A.GetSidSubAuthorityCount.restype = ctypes.POINTER(ctypes.c_ubyte)
A.GetSidSubAuthority.argtypes = [W.LPVOID, W.DWORD]
A.GetSidSubAuthority.restype = ctypes.POINTER(W.DWORD)
A.DuplicateToken.argtypes = [W.HANDLE, ctypes.c_int, ctypes.POINTER(W.HANDLE)]
A.CheckTokenMembership.argtypes = [W.HANDLE, W.LPVOID, ctypes.POINTER(W.BOOL)]
A.CreateProcessAsUserW.argtypes = [W.HANDLE, W.LPCWSTR, W.LPWSTR, W.LPVOID, W.LPVOID, W.BOOL,
                                 W.DWORD, W.LPVOID, W.LPCWSTR, ctypes.POINTER(Startup), ctypes.POINTER(Process)]


def require(value):
    if not value:
        raise ctypes.WinError(ctypes.get_last_error())


def elevation(token):
    value, size = W.DWORD(), W.DWORD()
    require(A.GetTokenInformation(token, 20, ctypes.byref(value), ctypes.sizeof(value), ctypes.byref(size)))
    return bool(value.value)


def verify_standard_token(token):
    size = W.DWORD()
    A.GetTokenInformation(token, 25, None, 0, ctypes.byref(size))
    data = ctypes.create_string_buffer(size.value)
    require(A.GetTokenInformation(token, 25, data, size, ctypes.byref(size)))
    sid = ctypes.cast(data, ctypes.POINTER(SidAttributes)).contents.sid
    authority = A.GetSidSubAuthority(sid, A.GetSidSubAuthorityCount(sid)[0] - 1)[0]
    admin_sid, impersonation, member = W.LPVOID(), W.HANDLE(), W.BOOL()
    try:
        require(A.ConvertStringSidToSidW('S-1-5-32-544', ctypes.byref(admin_sid)))
        require(A.DuplicateToken(token, 2, ctypes.byref(impersonation)))
        require(A.CheckTokenMembership(impersonation, admin_sid, ctypes.byref(member)))
        if elevation(token) or member.value or authority > 8192:
            raise AssertionError('Installed-app test token must be non-elevated, non-admin and at most medium integrity')
    finally:
        if impersonation: K.CloseHandle(impersonation)
        if admin_sid: K.LocalFree(admin_sid)


def main():
    if len(sys.argv) != 3:
        raise SystemExit('usage: windows-standard-user-smoke.py ABSOLUTE_NODE.exe ABSOLUTE_INSTALLED_APP.exe')
    probe = sys.argv[1] == '--probe'
    node = Path(sys.argv[2] if probe else sys.argv[1]).resolve(strict=True)
    app = None if probe else Path(sys.argv[2]).resolve(strict=True)
    if node.name.lower() != 'node.exe' or (app is not None and app.name.lower() != 'jv-desktop.exe'):
        raise AssertionError('Only the fixed installed-app acceptance command is allowed')
    argv = ([str(node), '-e', 'console.log("PASS standard-user launcher bootstrap")'] if probe else
            [str(node), str(ROOT / 'scripts/windows-webview-acceptance.mjs'), str(app), '--package-smoke'])
    original, reduced, job, medium = W.HANDLE(), W.HANDLE(), W.HANDLE(), W.LPVOID()
    process = Process()
    standard_handles = []
    try:
        require(A.OpenProcessToken(K.GetCurrentProcess(), 0x8b, ctypes.byref(original)))
        elevated = elevation(original)
        if os.environ.get('GITHUB_ACTIONS') == 'true':
            print(f'::notice title=Windows test privileges::Runner elevated: {str(elevated).lower()}', flush=True)
        if not elevated:
            verify_standard_token(original)
            print('PASS installed-app test uses ordinary Windows privileges', flush=True)
            return subprocess.run(argv, cwd=ROOT, shell=False, timeout=85).returncode
        size = W.DWORD()
        if A.GetTokenInformation(original, 19, ctypes.byref(reduced), ctypes.sizeof(reduced), ctypes.byref(size)):
            method = 'Windows linked standard-user token'
        else:
            require(A.CreateRestrictedToken(original, 0x5, 0, None, 0, None, 0, None, ctypes.byref(reduced)))
            require(A.ConvertStringSidToSidW('S-1-16-8192', ctypes.byref(medium)))
            label = SidAttributes(medium, 0x20)
            require(A.SetTokenInformation(reduced, 25, ctypes.byref(label), ctypes.sizeof(label) + A.GetLengthSid(medium)))
            method = 'restricted medium-integrity token'
        verify_standard_token(reduced)
        print(f'::notice title=Installed-app privileges::Verified non-elevated non-admin medium-integrity test token: {method}', flush=True)
        job = W.HANDLE(K.CreateJobObjectW(None, None))
        require(job)
        limits = JobLimits()
        limits.basic.flags = 0x2000  # KILL_ON_JOB_CLOSE; no breakaway permission.
        require(K.SetInformationJobObject(job, 9, ctypes.byref(limits), ctypes.sizeof(limits)))
        startup = Startup()
        startup.cb, startup.flags = ctypes.sizeof(startup), 0x100
        # Python's stdio handles need explicit inheritable duplicates when using
        # CreateProcessAsUser, rather than subprocess's own handle preparation.
        for stream in (sys.stdin, sys.stdout, sys.stderr):
            handle = W.HANDLE()
            require(K.DuplicateHandle(K.GetCurrentProcess(), msvcrt.get_osfhandle(stream.fileno()),
                                      K.GetCurrentProcess(), ctypes.byref(handle), 0, True, 2))
            standard_handles.append(handle)
        startup.stdin, startup.stdout, startup.stderr = standard_handles
        command = ctypes.create_unicode_buffer(subprocess.list2cmdline(argv))
        require(A.CreateProcessAsUserW(reduced, str(node), command, None, None, True, 4,
                                      None, str(ROOT), ctypes.byref(startup), ctypes.byref(process)))
        # Assignment happens while suspended: all test descendants belong to us.
        require(K.AssignProcessToJobObject(job, process.process))
        if K.ResumeThread(process.thread) == 0xffffffff:
            raise ctypes.WinError(ctypes.get_last_error())
        if K.WaitForSingleObject(process.process, 85000) != 0:
            raise TimeoutError('Standard-user installed WebView test exceeded its deadline')
        code = W.DWORD()
        require(K.GetExitCodeProcess(process.process, ctypes.byref(code)))
        if code.value:
            print(f'::error title=Standard-user test process::Child exit code: {code.value}', flush=True)
        return code.value
    finally:
        if process.process and K.WaitForSingleObject(process.process, 0) != 0:
            K.TerminateProcess(process.process, 1)
        if job: K.CloseHandle(job)
        for handle in (*standard_handles, process.thread, process.process, reduced, original):
            if handle: K.CloseHandle(handle)
        if medium: K.LocalFree(medium)


if __name__ == '__main__':
    try:
        sys.exit(main())
    except Exception as error:
        if os.environ.get('GITHUB_ACTIONS') == 'true':
            print(f'::error title=Standard-user WebView test::{type(error).__name__}; Windows code: {getattr(error, "winerror", None)}', flush=True)
        raise
