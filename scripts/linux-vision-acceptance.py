#!/usr/bin/env python3
"""Actual Linux Tauri GUI + source CLI + synthetic loopback vision acceptance.

Run inside a dedicated dbus-run-session and xvfb-run. Uses existing system AT-SPI,
Pillow and repository-local Playwright. No fixture feature or production service.
"""
import argparse
import ctypes
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import uuid

import pyatspi
import gi
gi.require_version('Gdk','3.0')
from gi.repository import Gdk

ROOT=Path(__file__).resolve().parents[1]


def walk(obj):
    yield obj
    try:
        children=list(obj)
    except Exception:
        return
    for child in children:
        if child:
            yield from walk(child)


def app():
    return next(o for o in pyatspi.Registry.getDesktop(0) if o.name=='jv-desktop')


def find(name, role=None):
    return next(o for o in walk(app()) if o.name==name and (role is None or o.getRoleName()==role))


def wait(check, timeout=30):
    end=time.monotonic()+timeout
    last=None
    while time.monotonic()<end:
        try:
            result=check()
            if result: return result
        except (StopIteration,RuntimeError,LookupError) as exc:
            last=type(exc).__name__
        time.sleep(.2)
    raise AssertionError('GUI condition timed out: '+str(last))


def chord(*names):
    # AT-SPI KEY_PRESS takes hardware keycodes, not GDK keysyms. Translate on
    # this private X display and release every modifier in reverse order.
    x=ctypes.CDLL('libX11.so.6'); xt=ctypes.CDLL('libXtst.so.6')
    x.XOpenDisplay.restype=ctypes.c_void_p
    x.XKeysymToKeycode.argtypes=[ctypes.c_void_p,ctypes.c_ulong]
    x.XKeysymToKeycode.restype=ctypes.c_uint
    x.XFlush.argtypes=[ctypes.c_void_p];x.XCloseDisplay.argtypes=[ctypes.c_void_p]
    xt.XTestFakeKeyEvent.argtypes=[ctypes.c_void_p,ctypes.c_uint,ctypes.c_int,ctypes.c_ulong]
    display=x.XOpenDisplay(None);assert display
    codes=[x.XKeysymToKeycode(display,Gdk.keyval_from_name(name)) for name in names]
    try:
        for code in codes: xt.XTestFakeKeyEvent(display,code,1,0)
        for code in reversed(codes): xt.XTestFakeKeyEvent(display,code,0,0)
        x.XFlush(display)
    finally: x.XCloseDisplay(display)
    time.sleep(.3)


def mouse(obj):
    box=obj.queryComponent().getExtents(pyatspi.DESKTOP_COORDS)
    x=ctypes.CDLL('libX11.so.6'); xt=ctypes.CDLL('libXtst.so.6')
    x.XOpenDisplay.restype=ctypes.c_void_p
    x.XCloseDisplay.argtypes=[ctypes.c_void_p]
    xt.XTestFakeMotionEvent.argtypes=[ctypes.c_void_p,ctypes.c_int,ctypes.c_int,ctypes.c_int,ctypes.c_ulong]
    xt.XTestFakeButtonEvent.argtypes=[ctypes.c_void_p,ctypes.c_uint,ctypes.c_int,ctypes.c_ulong]
    display=x.XOpenDisplay(None); assert display
    xt.XTestFakeMotionEvent(display,-1,box.x+box.width//2,box.y+box.height//2,0)
    xt.XTestFakeButtonEvent(display,1,1,0); xt.XTestFakeButtonEvent(display,1,0,0)
    x.XCloseDisplay(display); time.sleep(.3)


def focus_window(title):
    # Only windows in this driver's private Xvfb display are considered.
    tree=subprocess.check_output(['xwininfo','-root','-tree'],text=True)
    for line in tree.splitlines():
        if '"'+title+'"' not in line: continue
        wid=line.strip().split()[0]
        info=subprocess.check_output(['xwininfo','-id',wid],text=True)
        if 'Map State: IsViewable' not in info: continue
        x=ctypes.CDLL('libX11.so.6'); x.XOpenDisplay.restype=ctypes.c_void_p
        x.XSetInputFocus.argtypes=[ctypes.c_void_p,ctypes.c_ulong,ctypes.c_int,ctypes.c_ulong]
        x.XRaiseWindow.argtypes=[ctypes.c_void_p,ctypes.c_ulong]
        x.XCloseDisplay.argtypes=[ctypes.c_void_p]
        display=x.XOpenDisplay(None); assert display
        x.XRaiseWindow(display,int(wid,16)); x.XSetInputFocus(display,int(wid,16),2,0)
        x.XCloseDisplay(display); time.sleep(.3)
        return True
    return False


def choose(name,index):
    print('Select:',name,flush=True)
    obj=wait(lambda:find(name))
    obj.queryAction().doAction(0)
    options=[o for o in walk(obj) if o.getRoleName()=='menu item']
    assert options[index].queryAction().doAction(0)
    time.sleep(.3)


def click(name):
    print('Click:',name,flush=True)
    obj=wait(lambda:find(name,'push button'))
    assert obj.queryAction().doAction(0)


def fill(name,text):
    print('Fill:',name,flush=True)
    candidates=[o for o in walk(app()) if o.name==name or o.name.startswith(name+' ')]
    fields=[o for candidate in candidates for o in walk(candidate)
            if o.getState().contains(pyatspi.STATE_EDITABLE)]
    if not fields: fields=candidates
    assert fields, 'Input unavailable: '+name
    obj=fields[0]
    obj.queryComponent().grabFocus(); time.sleep(.2)
    mouse(obj); chord('Control_L','a')
    x=ctypes.CDLL('libX11.so.6'); xt=ctypes.CDLL('libXtst.so.6')
    x.XOpenDisplay.restype=ctypes.c_void_p
    x.XKeysymToKeycode.argtypes=[ctypes.c_void_p,ctypes.c_ulong]
    x.XKeysymToKeycode.restype=ctypes.c_uint
    x.XCloseDisplay.argtypes=[ctypes.c_void_p]
    xt.XTestFakeKeyEvent.argtypes=[ctypes.c_void_p,ctypes.c_uint,ctypes.c_int,ctypes.c_ulong]
    display=x.XOpenDisplay(None); assert display
    shift=x.XKeysymToKeycode(display,Gdk.keyval_from_name('Shift_L'))
    try:
        for char in text:
            assert char.isascii()
            shifted=char.isupper() or char in ':_+!?@#$%^&*(){}|"<>'
            code=x.XKeysymToKeycode(display,Gdk.unicode_to_keyval(ord(char)))
            if shifted: xt.XTestFakeKeyEvent(display,shift,1,0)
            xt.XTestFakeKeyEvent(display,code,1,0); xt.XTestFakeKeyEvent(display,code,0,0)
            if shifted: xt.XTestFakeKeyEvent(display,shift,0,0)
    finally: x.XCloseDisplay(display)
    time.sleep(.3)


def texts():
    out=[]
    for o in walk(app()):
        if o.name:
            out.append(o.name)
        try:
            value=o.queryText().getText(0,-1)
            if value: out.append(value)
        except (NotImplementedError,RuntimeError):
            pass
    return '\n'.join(out)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cli-root',type=Path,required=True)
    parser.add_argument('--app',type=Path,default=ROOT/'target/release/jv-desktop')
    args=parser.parse_args()
    assert sys.platform=='linux'
    cli=args.cli_root.resolve()
    spec=importlib.util.spec_from_file_location('vision_fixture',cli/'scripts/vision_bridge_smoke.py')
    module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    folder=ROOT/'.cache'/('linux-vision-gui-'+uuid.uuid4().hex);folder.mkdir(parents=True,mode=0o700)
    workspace=Path(tempfile.mkdtemp(prefix='jv-desktop-vision-workspace-'))
    fixture=module.Fixture(workspace,ROOT/'.tools/node/bin/node',ROOT/'node_modules/playwright',ROOT/'.cache/ms-playwright')
    env={**os.environ,'PATH':str(cli/'bin')+os.pathsep+os.environ['PATH'],
        'HOME':str(folder/'home'),'XDG_CONFIG_HOME':str(folder/'config'),
        'XDG_DATA_HOME':str(folder/'data'),'XDG_CACHE_HOME':str(folder/'cache'),
        'GTK_MODULES':'gail:atk-bridge','NO_AT_BRIDGE':'0','WEBKIT_FORCE_SANDBOX':'1'}
    for key_ in ('HOME','XDG_CONFIG_HOME','XDG_DATA_HOME','XDG_CACHE_HOME'):
        Path(env[key_]).mkdir(mode=0o700)
    log=(folder/'application.log').open('wb')
    wm=subprocess.Popen(['xfwm4','--compositor=off'],env=env,stdout=log,stderr=log)
    process=subprocess.Popen([str(args.app.resolve())],env=env,stdout=log,stderr=log)
    try:
        print('Wait for native application',flush=True)
        wait(app)
        choose('Backend mode',1)
        wait(lambda:'JV CLI: detected' in texts())
        click('Open Folder')
        # Native GTK picker: Ctrl+L opens its explicit location entry.
        wait(lambda:focus_window('Open a project folder'))
        home=find('Home','label')
        mouse(home)
        time.sleep(.5)
        chord('Control_L','l')
        entry=wait(lambda:next(o for o in walk(app()) if o.getRoleName()=='text' and o.getState().contains(pyatspi.STATE_FOCUSED)))
        entry.queryComponent().grabFocus()
        entry.queryEditableText().setTextContents(str(workspace)+'/')
        time.sleep(.5)
        entry.queryComponent().grabFocus()
        action=entry.queryAction()
        assert action.doAction(0)
        time.sleep(2)
        # Some GTK versions navigate first and require Open to confirm.
        try: find('Open','push button').queryAction().doAction(0)
        except StopIteration: pass
        wait(lambda:find('Username'))
        fill('Username','synthetic');fill('Service URL',fixture.url);fill('Password',module.PASSWORD)
        choose('Workspace permission',1)
        # Enable tool networking for the local browser's own process sockets.
        network=find('Tool network','check box')
        if not network.getState().contains(pyatspi.STATE_CHECKED): network.queryAction().doAction(0)
        assert any(o.name=='Service URL '+fixture.url for o in walk(app())), 'Loopback URL was not entered'
        click('Sign in')
        wait(lambda:'Structured session' in texts())
        for n in range(2):
            fill('Message JV','Run the synthetic website screenshot inspection and follow-up edit.' if n==0 else 'Verify the previous edited website.')
            click('Send message')
            wait(lambda:fixture.completed_tasks==n+1,timeout=180)
            wait(lambda:'Turn complete' in texts())
        assert fixture.completed_tasks==2 and len(fixture.images)==3 and not fixture.errors
        assert 'After inspection' in (workspace/'dist/index.html').read_text()
        visible=texts()
        assert 'VISION_BRIDGE_OK' in visible
        for secret in (module.PASSWORD,module.TOKEN,'base64,','image_url'): assert secret not in visible
        # Unsupported service response is a turn failure; session stays open.
        fixture.mode='unsupported'
        fill('Message JV','Synthetic unsupported request');click('Send message')
        wait(lambda:'JV-AGENT-UNSUPPORTED-001' in texts())
        wait(lambda:'Structured session' in texts() and 'Turn did not complete' in texts())
        assert not any(o.name=='Sign in' for o in walk(app()))
        from PIL import ImageGrab
        ImageGrab.grab().save(folder/'completed-gui.png')
        assert fixture.legacy_posts==0
        # Exit using the native window action, not a process kill.
        assert focus_window('JV Desktop')
        chord('Alt_L','F4')
        process.wait(timeout=20)
        log.flush()
        raw=(folder/'application.log').read_bytes()
        for secret in (module.PASSWORD,module.TOKEN,'base64,','image_url'): assert secret.encode() not in raw
        report={'ok':True,'actual_tauri_gui':True,'synthetic_provider':True,'live_inference':False,
            'native_picker':True,'structured_turns':2,'images':3,'unsupported_keeps_session':True,
            'no_automatic_fallback':True,'native_close':True,'workspace':str(workspace)}
        (folder/'report.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(report,indent=2)); print('Evidence:',folder)
    except Exception:
        print('Acceptance failed; private evidence:',folder,file=sys.stderr)
        raise
    finally:
        if process.poll() is None:
            process.terminate()
            try:process.wait(timeout=20)
            except subprocess.TimeoutExpired:process.kill();process.wait()
        log.close();fixture.close()
        wm.terminate();wm.wait(timeout=10)


if __name__=='__main__': main()
