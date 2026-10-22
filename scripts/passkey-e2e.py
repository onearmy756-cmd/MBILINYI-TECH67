#!/usr/bin/env python3
"""End-to-end passkey test against a real Chromium with a virtual authenticator.

Chromium's CDP exposes WebAuthn.addVirtualAuthenticator, which behaves like a
real platform authenticator: it holds a private key, requires a user-presence
gesture, and reports a signature counter. That makes this a genuine test of the
whole ceremony rather than a mock of it.

Run it against a server you already have up:

    BASE=http://localhost:12000 python3 scripts/passkey-e2e.py

Chromium only allows WebAuthn on a "secure context", so the host must be
localhost or an https origin — 127.0.0.1 is rejected as an invalid domain.

Signing in or out regenerates the session id, and with it the CSRF token, so the
test reloads the page after every such step — exactly what the real UI does when
it redirects to the portal.
"""
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.request

try:
    import websocket
except ImportError:
    sys.exit("Missing dependency: pip install websocket-client")

BASE = os.environ.get("BASE", "http://localhost:12000")
PORT = int(os.environ.get("CDP_PORT", "9333"))
CHROME = os.environ.get("CHROME", "chromium")
ok = 0
fail = 0


def check(cond, msg):
    global ok, fail
    if cond:
        ok += 1
        print("  OK   " + msg)
    else:
        fail += 1
        print("  FAIL " + msg)


profile = tempfile.mkdtemp(prefix="passkey-e2e-")
chrome = subprocess.Popen(
    [
        CHROME, "--headless=new", "--remote-debugging-port=%d" % PORT,
        "--user-data-dir=" + profile, "--no-sandbox", "--disable-gpu",
        "--no-first-run", "--disable-dev-shm-usage", "--remote-allow-origins=*",
        "about:blank",
    ],
    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
)

ws = None
try:
    page = None
    for _ in range(60):
        try:
            tabs = json.load(urllib.request.urlopen("http://127.0.0.1:%d/json" % PORT))
            page = [t for t in tabs if t["type"] == "page"]
            if page:
                break
        except Exception:
            pass
        time.sleep(0.5)
    if not page:
        sys.exit("Chromium did not start")

    ws = websocket.create_connection(page[0]["webSocketDebuggerUrl"], timeout=30)
    counter = {"id": 0}

    def cmd(method, **params):
        counter["id"] += 1
        mid = counter["id"]
        ws.send(json.dumps({"id": mid, "method": method, "params": params}))
        while True:
            msg = json.loads(ws.recv())
            if msg.get("id") == mid:
                if "error" in msg:
                    raise RuntimeError("%s: %s" % (method, msg["error"]))
                return msg.get("result", {})

    def ev(expr):
        r = cmd("Runtime.evaluate", expression=expr, awaitPromise=True, returnByValue=True)
        if "exceptionDetails" in r:
            det = r["exceptionDetails"]
            raise RuntimeError(det.get("exception", {}).get("description", "JS error"))
        return r.get("result", {}).get("value")

    def reload_page():
        cmd("Page.navigate", url=BASE + "/auth")
        time.sleep(2)

    cmd("Runtime.enable")
    cmd("Page.enable")
    cmd("WebAuthn.enable")
    auth = cmd("WebAuthn.addVirtualAuthenticator", options={
        "protocol": "ctap2",
        "transport": "internal",
        "hasResidentKey": True,
        "hasUserVerification": True,
        "isUserVerified": True,
        "automaticPresenceSimulation": True,
    })["authenticatorId"]
    print("  virtual authenticator: %s...\n" % auth[:8])

    print("=== 1. the page loads and the JS is present ===")
    reload_page()
    check(ev("typeof doPasskeyLogin === 'function'"), "doPasskeyLogin is defined")
    check(ev("typeof doPasskeyRegister === 'function'"), "doPasskeyRegister is defined")
    check(ev("waSupported()") is True, "waSupported() == true")
    check(ev("!$('passkeyLoginWrap').classList.contains('hidden')"),
          "the passkey button is visible")

    print("\n=== 2. register a fresh account ===")
    email = "pk.%d@example.com" % int(time.time())
    ev("(async () => {"
       "  const d = await apiPost('api/auth.php', {action:'register', name:'PK Test',"
       "    phone:'0712345999', email:" + json.dumps(email) + ", password:'secret123',"
       "    password2:'secret123', pin:'4321', pin2:'4321'});"
       "  window.__ME__ = d.user; return d.user.email; })()")
    reload_page()
    check(ev("window.__ME__ && window.__ME__.email") == email,
          "account created and session is live: " + email)

    print("\n=== 3. register a passkey for this device ===")
    reg = ev("""(async () => {
      try {
        const begin = await apiPost('api/auth.php', {action:'passkey-register-begin'});
        const cred = await navigator.credentials.create({publicKey:{
          challenge: b64uToBuf(begin.challenge), rp: begin.rp,
          user:{id:b64uToBuf(begin.user.id),name:begin.user.name,displayName:begin.user.displayName},
          pubKeyCredParams: begin.pubKeyCredParams, timeout: begin.timeout,
          attestation: begin.attestation, authenticatorSelection: begin.authenticatorSelection,
          excludeCredentials: (begin.excludeCredentials||[]).map(c=>({type:'public-key',id:b64uToBuf(c.id),transports:c.transports})),
        }});
        const transports = cred.response.getTransports ? cred.response.getTransports() : [];
        const d = await apiPost('api/auth.php', {action:'passkey-register-finish', label:'Chromium', transports,
          response:{clientDataJSON:bufToB64u(cred.response.clientDataJSON),
                    attestationObject:bufToB64u(cred.response.attestationObject)}});
        return {ok:true, label:d.label};
      } catch (e) { return {ok:false, err:(e&&e.message)||String(e)}; }
    })()""")
    check(reg and reg.get("ok"), "passkey registered -> %s" % reg)

    print("\n=== 4. the passkey is listed ===")
    lst = ev("(async()=>{const d=await apiGet('api/auth.php',{action:'passkey-list'});return d.passkeys.length;})()")
    check(lst == 1, "exactly one passkey shown (%s)" % lst)

    print("\n=== 5. sign out, then sign in with the passkey ALONE ===")
    ev("(async()=>{await apiPost('api/auth.php',{action:'logout'}); return true;})()")
    reload_page()
    check(ev("window.__ME__") is None, "signed out (server reports no user)")
    ev("$('loginEmail').value = " + json.dumps(email))
    login = ev("""(async () => {
      try {
        const begin = await apiPost('api/auth.php', {action:'passkey-login-begin', email:$('loginEmail').value});
        const pk = {challenge:b64uToBuf(begin.challenge), rpId:begin.rpId, timeout:begin.timeout,
                    userVerification:begin.userVerification};
        if (begin.allowCredentials.length) pk.allowCredentials = begin.allowCredentials.map(
          c=>({type:'public-key',id:b64uToBuf(c.id),transports:c.transports}));
        const cred = await navigator.credentials.get({publicKey: pk});
        const d = await apiPost('api/auth.php', {action:'passkey-login-finish', id:bufToB64u(cred.rawId),
          response:{clientDataJSON:bufToB64u(cred.response.clientDataJSON),
                    authenticatorData:bufToB64u(cred.response.authenticatorData),
                    signature:bufToB64u(cred.response.signature)}});
        return {ok:true, email:d.user.email, role:d.user.role};
      } catch (e) { return {ok:false, err:(e&&e.message)||String(e)}; }
    })()""")
    check(login and login.get("ok"), "passkey sign-in succeeded -> %s" % login)
    check(login and login.get("email") == email, "it is the right user")
    reload_page()
    check(ev("window.__ME__ && window.__ME__.email") == email,
          "server-side session confirmed after sign-in")

    print("\n=== 6. rejection: a made-up credential ===")
    bad = ev("""(async () => {
      try {
        await apiPost('api/auth.php', {action:'passkey-login-begin', email:''});
        await apiPost('api/auth.php', {action:'passkey-login-finish', id:'bm90LXJlYWw',
          response:{clientDataJSON:'e30',authenticatorData:'e30',signature:'e30'}});
        return {ok:false, err:'it was NOT rejected'};
      } catch (e) { return {ok:true, err:(e&&e.message)||String(e)}; }
    })()""")
    check(bad and bad.get("ok"), "made-up credential rejected -> %s" % (bad.get("err") if bad else bad))

    print("\n=== 7. a second device, then delete one passkey ===")
    ev("(async()=>{await apiPost('api/auth.php',{action:'login',email:" + json.dumps(email) +
       ",password:'secret123'});"
       "await apiPost('api/auth.php',{action:'verify-login-pin',email:" + json.dumps(email) + ",pin:'4321'});"
       "return true;})()")
    reload_page()
    n2 = ev("""(async () => {
      const d = await apiPost('api/auth.php', {action:'passkey-register-begin'});
      const c = await navigator.credentials.create({publicKey:{
        challenge:b64uToBuf(d.challenge), rp:d.rp,
        user:{id:b64uToBuf(d.user.id),name:d.user.name,displayName:d.user.displayName},
        pubKeyCredParams:d.pubKeyCredParams, timeout:d.timeout, attestation:d.attestation,
        authenticatorSelection:d.authenticatorSelection, excludeCredentials:[]}});
      await apiPost('api/auth.php', {action:'passkey-register-finish', label:'Second', transports:[],
        response:{clientDataJSON:bufToB64u(c.response.clientDataJSON),
                  attestationObject:bufToB64u(c.response.attestationObject)}});
      const l = await apiGet('api/auth.php', {action:'passkey-list'});
      return l.passkeys.length;
    })()""")
    check(n2 == 2, "second passkey registered (total %s)" % n2)

    left = ev("""(async () => {
      const l = await apiGet('api/auth.php', {action:'passkey-list'});
      await apiPost('api/auth.php', {action:'passkey-delete', id:l.passkeys[0].id});
      const after = await apiGet('api/auth.php', {action:'passkey-list'});
      return after.passkeys.length;
    })()""")
    check(left == 1, "one passkey deleted (%s left)" % left)

    print("\nTOTAL: %d passed, %d failed" % (ok, fail))
finally:
    if ws:
        try:
            ws.close()
        except Exception:
            pass
    chrome.terminate()
    shutil.rmtree(profile, ignore_errors=True)

sys.exit(0 if fail == 0 else 1)
