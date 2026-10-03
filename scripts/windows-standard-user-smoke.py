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
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
if os.name != 'nt':
    raise SystemExit('Native Windows required')
K = ctypes.WinDLL('kernel32', use_last_error=True)
A = ctypes.WinDLL('advapi32', use_last_error=True)


class SidAttributes(ctypes.Structure):
    _fields_ = [('sid', W.LPVOID), ('attributes', W.DWORD)]


class Trustee(ctypes.Structure):
    _fields_ = [('multiple', W.LPVOID), ('operation', ctypes.c_int),
                ('form', ctypes.c_int), ('kind', ctypes.c_int), ('sid', W.LPVOID)]


class AccessEntry(ctypes.Structure):
    _fields_ = [('permissions', W.DWORD), ('mode', ctypes.c_int),
                ('inheritance', W.DWORD), ('trustee', Trustee)]


class AclSize(ctypes.Structure):
    _fields_ = [('count', W.DWORD), ('used', W.DWORD), ('free', W.DWORD)]


class SecurityAttributes(ctypes.Structure):
    _fields_ = [('length', W.DWORD), ('descriptor', W.LPVOID), ('inherit', W.BOOL)]


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
K.CreatePipe.argtypes = [ctypes.POINTER(W.HANDLE), ctypes.POINTER(W.HANDLE), ctypes.POINTER(SecurityAttributes), W.DWORD]
K.SetHandleInformation.argtypes = [W.HANDLE, W.DWORD, W.DWORD]
K.PeekNamedPipe.argtypes = [W.HANDLE, W.LPVOID, W.DWORD, W.LPVOID, ctypes.POINTER(W.DWORD), W.LPVOID]
K.ReadFile.argtypes = [W.HANDLE, W.LPVOID, W.DWORD, ctypes.POINTER(W.DWORD), W.LPVOID]
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
A.DuplicateTokenEx.argtypes = [W.HANDLE, W.DWORD, W.LPVOID, ctypes.c_int, ctypes.c_int, ctypes.POINTER(W.HANDLE)]
A.CheckTokenMembership.argtypes = [W.HANDLE, W.LPVOID, ctypes.POINTER(W.BOOL)]
A.GetSecurityInfo.argtypes = [W.HANDLE, ctypes.c_int, W.DWORD, W.LPVOID, W.LPVOID,
                              ctypes.POINTER(W.LPVOID), W.LPVOID, ctypes.POINTER(W.LPVOID)]
A.SetSecurityInfo.argtypes = [W.HANDLE, ctypes.c_int, W.DWORD, W.LPVOID, W.LPVOID, W.LPVOID, W.LPVOID]
A.SetEntriesInAclW.argtypes = [W.DWORD, ctypes.POINTER(AccessEntry), W.LPVOID, ctypes.POINTER(W.LPVOID)]
A.GetAclInformation.argtypes = [W.LPVOID, W.LPVOID, W.DWORD, ctypes.c_int]
A.GetAce.argtypes = [W.LPVOID, W.DWORD, ctypes.POINTER(W.LPVOID)]
A.EqualSid.argtypes = [W.LPVOID, W.LPVOID]
A.CreateProcessAsUserW.argtypes = [W.HANDLE, W.LPCWSTR, W.LPWSTR, W.LPVOID, W.LPVOID, W.BOOL,
                                 W.DWORD, W.LPVOID, W.LPCWSTR, ctypes.POINTER(Startup), ctypes.POINTER(Process)]
K.CreateProcessW.argtypes = A.CreateProcessAsUserW.argtypes[1:]


def require(value):
    if not value:
        raise ctypes.WinError(ctypes.get_last_error())


def elevation(token):
    value, size = W.DWORD(), W.DWORD()
    require(A.GetTokenInformation(token, 20, ctypes.byref(value), ctypes.sizeof(value), ctypes.byref(size)))
    return bool(value.value)


def token_data(token, kind):
    size = W.DWORD()
    A.GetTokenInformation(token, kind, None, 0, ctypes.byref(size))
    if not 0 < size.value <= 65536:
        raise AssertionError('Unexpected token information size')
    data = ctypes.create_string_buffer(size.value)
    require(A.GetTokenInformation(token, kind, data, size, ctypes.byref(size)))
    return data


def user_grant(acl, sid, mask):
    if not acl:
        raise AssertionError('A null token ACL is not allowed')
    size = AclSize()
    require(A.GetAclInformation(acl, ctypes.byref(size), ctypes.sizeof(size), 2))
    for index in range(size.count):
        ace = W.LPVOID()
        require(A.GetAce(acl, index, ctypes.byref(ace)))
        if ctypes.c_ubyte.from_address(ace.value).value == 0:
            granted = W.DWORD.from_address(ace.value + 4).value
            if A.EqualSid(ace.value + 8, sid) and (granted & mask == mask or granted & 0x10000000):
                return True
    return False


def grant_test_token_user(token):
    """Grant only this account access to the newly copied test token and objects.

    An elevated runner's token DACL can grant Administrators but omit its user.
    After that group becomes deny-only, descendants cannot reopen their own
    primary token. Never modify the original/linked login token or any file ACL.
    """
    user_data = token_data(token, 1)
    sid = ctypes.cast(user_data, ctypes.POINTER(SidAttributes)).contents.sid
    descriptor, object_acl, updated = W.LPVOID(), W.LPVOID(), W.LPVOID()
    try:
        error = A.GetSecurityInfo(token, 6, 4, None, None, ctypes.byref(object_acl), None, ctypes.byref(descriptor))
        if error: raise ctypes.WinError(error)
        default_data = token_data(token, 6)  # TokenDefaultDacl
        default_acl = ctypes.cast(default_data, ctypes.POINTER(W.LPVOID)).contents.value
        if os.environ.get('GITHUB_ACTIONS') == 'true':
            print('::notice title=Installed-app privileges::Current-user token grants before repair: object ' +
                  str(user_grant(object_acl, sid, 0xf01ff)).lower() + '; default ' +
                  str(user_grant(default_acl, sid, 0x10000000)).lower(), flush=True)
        entry = AccessEntry(0xf01ff, 1, 0, Trustee(None, 0, 0, 1, sid))
        error = A.SetEntriesInAclW(1, ctypes.byref(entry), object_acl, ctypes.byref(updated))
        if error: raise ctypes.WinError(error)
        error = A.SetSecurityInfo(token, 6, 4, None, None, updated, None)
        if error: raise ctypes.WinError(error)
        assert user_grant(updated, sid, 0xf01ff)
        K.LocalFree(updated); updated = W.LPVOID()
        entry.permissions = 0x10000000  # GENERIC_ALL for newly created objects.
        error = A.SetEntriesInAclW(1, ctypes.byref(entry), default_acl, ctypes.byref(updated))
        if error: raise ctypes.WinError(error)
        value = W.LPVOID(updated.value)
        require(A.SetTokenInformation(token, 6, ctypes.byref(value), ctypes.sizeof(value)))
        assert user_grant(updated, sid, 0x10000000)
    finally:
        if updated: K.LocalFree(updated)
        if descriptor: K.LocalFree(descriptor)


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
    direct_app = sys.argv[1] == '--app'
    probe = sys.argv[1] == '--probe'
    node = None if direct_app else Path(sys.argv[2] if probe else sys.argv[1]).resolve(strict=True)
    app = None if probe else Path(sys.argv[2]).resolve(strict=True)
    if (node is not None and node.name.lower() != 'node.exe') or (app is not None and app.name.lower() != 'jv-desktop.exe'):
        raise AssertionError('Only the fixed installed-app acceptance command is allowed')
    # Catch early module/cache failures before the WebView driver's own try block.
    # Emit only error categories, never paths, environment values or raw stderr.
    failure = '''catch(error) {
      const category = value => /^[A-Za-z0-9_]+$/.test(value || '') ? value : 'unknown';
      console.error('::error title=Standard-user acceptance bootstrap::' +
        category(error.name) + '; code: ' + category(error.code) + '; operation: ' + category(error.syscall));
      process.exitCode = 1;
    }'''
    if direct_app:
        argv = [str(app)]
    elif probe:
        code = '''import fs from 'node:fs'; import {randomUUID} from 'node:crypto';
        import {spawnSync} from 'node:child_process';
        import {join} from 'node:path'; try {
          fs.readFileSync(process.argv[1]); fs.mkdirSync(process.argv[2], {recursive:true});
          const file = join(process.argv[2], 'bootstrap-' + randomUUID());
          fs.writeFileSync(file, 'synthetic launcher check', {flag:'wx'}); fs.unlinkSync(file);
          const child = spawnSync(process.execPath, ['-e','process.exit(0)'], {shell:false, timeout:3000, stdio:'ignore'});
          if (child.error) throw child.error;
          if (child.status !== 0) throw new Error('Child launch failed');
          console.log('PASS standard-user launcher bootstrap: source read/cache write/child launch');
        } ''' + failure
        argv = [str(node), '--input-type=module', '-e', code,
                str(ROOT / 'scripts/windows-webview-acceptance.mjs'), str(ROOT / '.cache/gui')]
    else:
        code = "import {pathToFileURL} from 'node:url'; try { await import(pathToFileURL(process.argv[1]).href); } " + failure
        argv = [str(node), '--input-type=module', '-e', code,
                str(ROOT / 'scripts/windows-webview-acceptance.mjs'), str(app), '--package-smoke']
    executable = str(app if direct_app else node)
    original, reduced, job, medium = W.HANDLE(), W.HANDLE(), W.HANDLE(), W.LPVOID()
    process = Process()
    standard_handles = []
    read_pipe, write_pipe = W.HANDLE(), W.HANDLE()
    try:
        require(A.OpenProcessToken(K.GetCurrentProcess(), 0x6008b, ctypes.byref(original)))
        elevated = elevation(original)
        if os.environ.get('GITHUB_ACTIONS') == 'true':
            print(f'::notice title=Windows test privileges::Runner elevated: {str(elevated).lower()}', flush=True)
        if not elevated:
            verify_standard_token(original)
            print('PASS installed-app test uses ordinary Windows privileges', flush=True)
            if not direct_app:
                return subprocess.run(argv, cwd=ROOT, shell=False, timeout=85).returncode
        else:
            size = W.DWORD()
            linked = W.HANDLE()
            if A.GetTokenInformation(original, 19, ctypes.byref(linked), ctypes.sizeof(linked), ctypes.byref(size)):
                try:
                    require(A.DuplicateTokenEx(linked, 0x6008b, None, 2, 1, ctypes.byref(reduced)))
                finally:
                    K.CloseHandle(linked)
                method = 'copied Windows linked standard-user token'
            else:
                require(A.CreateRestrictedToken(original, 0x5, 0, None, 0, None, 0, None, ctypes.byref(reduced)))
                require(A.ConvertStringSidToSidW('S-1-16-8192', ctypes.byref(medium)))
                label = SidAttributes(medium, 0x20)
                require(A.SetTokenInformation(reduced, 25, ctypes.byref(label), ctypes.sizeof(label) + A.GetLengthSid(medium)))
                method = 'restricted medium-integrity token'
            grant_test_token_user(reduced)
            verify_standard_token(reduced)
            print(f'::notice title=Installed-app privileges::Verified non-elevated non-admin medium-integrity test token: {method}', flush=True)
        job = W.HANDLE(K.CreateJobObjectW(None, None))
        require(job)
        limits = JobLimits()
        limits.basic.flags = 0x2000  # KILL_ON_JOB_CLOSE; no breakaway permission.
        require(K.SetInformationJobObject(job, 9, ctypes.byref(limits), ctypes.sizeof(limits)))
        startup = Startup()
        startup.cb, startup.flags = ctypes.sizeof(startup), 0x100
        # CreateProcessAsUser must select the interactive desktop explicitly.
        # This uses existing access; it never changes its ACL or machine policy.
        startup.desktop = 'winsta0\\default'
        # Python's stdio handles need explicit inheritable duplicates when using
        # CreateProcessAsUser, rather than subprocess's own handle preparation.
        for stream in (sys.stdin,):
            handle = W.HANDLE()
            require(K.DuplicateHandle(K.GetCurrentProcess(), msvcrt.get_osfhandle(stream.fileno()),
                                      K.GetCurrentProcess(), ctypes.byref(handle), 0, True, 2))
            standard_handles.append(handle)
        attributes = SecurityAttributes(ctypes.sizeof(SecurityAttributes), None, True)
        require(K.CreatePipe(ctypes.byref(read_pipe), ctypes.byref(write_pipe), ctypes.byref(attributes), 0))
        require(K.SetHandleInformation(read_pipe, 1, 0))
        startup.stdin = standard_handles[0]
        startup.stdout = startup.stderr = write_pipe
        command = ctypes.create_unicode_buffer(subprocess.list2cmdline(argv))
        create = A.CreateProcessAsUserW if elevated else K.CreateProcessW
        prefix = (reduced,) if elevated else ()
        require(create(*prefix, executable, command, None, None, True, 4,
                       None, str(ROOT), ctypes.byref(startup), ctypes.byref(process)))
        # Assignment happens while suspended: all test descendants belong to us.
        require(K.AssignProcessToJobObject(job, process.process))
        K.CloseHandle(write_pipe)
        write_pipe = W.HANDLE()
        if K.ResumeThread(process.thread) == 0xffffffff:
            raise ctypes.WinError(ctypes.get_last_error())
        output = bytearray()
        deadline = time.monotonic() + 85
        while True:
            available = W.DWORD()
            if K.PeekNamedPipe(read_pipe, None, 0, None, ctypes.byref(available), None):
                if available.value:
                    buffer = ctypes.create_string_buffer(min(available.value, 65537))
                    count = W.DWORD()
                    require(K.ReadFile(read_pipe, buffer, len(buffer), ctypes.byref(count), None))
                    output.extend(buffer.raw[:count.value])
                    if len(output) > 65536:
                        raise AssertionError('Standard-user test output exceeded 64 KiB')
                    continue
            elif ctypes.get_last_error() != 109:
                raise ctypes.WinError(ctypes.get_last_error())
            if K.WaitForSingleObject(process.process, 0) == 0:
                break
            if time.monotonic() >= deadline:
                raise TimeoutError('Standard-user installed WebView test exceeded its deadline')
            time.sleep(0.02)
        for line in output.decode('utf-8', errors='replace').splitlines():
            # This fixed offline command never signs in, starts a turn or prints
            # child stderr. Forward only its own assertions/static annotations.
            if not direct_app and line.startswith(('PASS ', '::error title=')):
                print(line, flush=True)
        code = W.DWORD()
        require(K.GetExitCodeProcess(process.process, ctypes.byref(code)))
        if code.value:
            decoded = output.decode('utf-8', errors='replace')
            known = [value for value in ('EACCES', 'EPERM', 'ENOENT', 'EINVAL', 'ERR_MODULE_NOT_FOUND', 'MODULE_NOT_FOUND', 'SyntaxError') if value in decoded]
            if direct_app:
                known += [category for marker, category in [('error while running tauri', 'TAURI_RUN_ERROR'), ('WebView', 'WEBVIEW'), ('Access is denied', 'ACCESS_DENIED')] if marker.lower() in decoded.lower()]
            native_codes = sorted(set(re.findall(r'0x[0-9a-fA-F]{8}', decoded)))[:4]
            print(f'::error title=Standard-user test process::Child exit code: {code.value}; categories: {",".join(known) or "unknown"}; native codes: {",".join(native_codes) or "none"}', flush=True)
        return code.value
    finally:
        if process.process and K.WaitForSingleObject(process.process, 0) != 0:
            K.TerminateProcess(process.process, 1)
        if job: K.CloseHandle(job)
        for handle in (read_pipe, write_pipe, *standard_handles, process.thread, process.process, reduced, original):
            if handle: K.CloseHandle(handle)
        if medium: K.LocalFree(medium)


if __name__ == '__main__':
    try:
        sys.exit(main())
    except Exception as error:
        if os.environ.get('GITHUB_ACTIONS') == 'true':
            print(f'::error title=Standard-user WebView test::{type(error).__name__}; Windows code: {getattr(error, "winerror", None)}', flush=True)
        raise
