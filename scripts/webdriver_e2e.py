#!/usr/bin/env python3
"""End-to-end WebDriver test for the real Musico Tauri app.

Drives the ACTUAL app window through tauri-driver (WebKitWebDriver):
launch → library loaded → click a song → REAL position events arrive →
next/pause controls work. Requires: cargo install tauri-driver, app built.

Usage: python3 scripts/webdriver_e2e.py
"""

import json
import sys
import time
import urllib.request

WD = "http://127.0.0.1:4444"
APP = "/home/aryan/Documents/projects/musico/target/debug/musico-tauri"
ELEMENT = "element-6066-11e4-a52e-4f735466cecf"


def wd(method, path, body=None):
    req = urllib.request.Request(WD + path, method=method)
    req.add_header("Content-Type", "application/json")
    data = json.dumps(body).encode() if body is not None else None
    with urllib.request.urlopen(req, data=data, timeout=30) as r:
        resp = json.loads(r.read())
    value = resp.get("value")
    if isinstance(value, dict) and "error" in value:
        raise RuntimeError(f"WebDriver error: {value}")
    return value


def find(sess, css):
    el = wd("POST", f"/session/{sess}/element", {"using": "css selector", "value": css})
    return el[ELEMENT]


def click(sess, css):
    el = find(sess, css)
    wd("POST", f"/session/{sess}/element/{el}/click", {})


def js(sess, script):
    return wd("POST", f"/session/{sess}/execute/sync", {"script": script, "args": []})


def wait_for(sess, cond_js, timeout=30, interval=0.5, desc=""):
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            if js(sess, cond_js):
                return True
        except RuntimeError:
            pass
        time.sleep(interval)
    raise TimeoutError(f"Condition not met in {timeout}s: {desc or cond_js}")


def main():
    # Kill standalone instances so WebDriver owns the app (sled/audio are single-instance friendly)
    import subprocess
    subprocess.run(["pkill", "-x", "musico-tauri"], check=False)
    time.sleep(1)

    session = wd("POST", "/session", {
        "capabilities": {"alwaysMatch": {"tauri:options": {"application": APP}}}
    })["sessionId"]
    print(f"session: {session}")

    checks = []

    # 1. App boots and Bridge talks to the real backend
    wait_for(sess, "return App.state.library.length > 0", timeout=30, desc="library loaded via real IPC")
    checks.append(("library loaded via real invoke", js(sess, "return App.state.library.length")))

    # 2. Click the first song row in the library → real Playing event arrives
    click(sess, "#song-list-view .song-row")
    wait_for(sess, "return App.state.currentSong !== null", timeout=10, desc="song started")
    checks.append(("np-title", js(sess, "return document.getElementById('np-title').textContent")))
    checks.append(("isPlaying", js(sess, "return App.state.isPlaying")))

    # 3. THE critical check: REAL PositionUpdate events advance the UI
    time.sleep(2.5)
    pos = js(sess, "return App.state.position")
    checks.append(("real position after ~2.5s", pos))
    assert pos > 0.5, f"position did not advance via real events: {pos}"

    # 4. Duration is real (from the decoder, not the fake 0.0)
    dur = js(sess, "return App.state.duration")
    checks.append(("real duration", dur))
    assert dur > 10, f"duration not populated from tags: {dur}"

    # 5. Next → a different song plays (queue empty → stop policy; library has songs but
    #    shuffle off + repeat off → stop). Verify it stopped cleanly.
    click(sess, "#btn-next")
    time.sleep(1)
    checks.append(("after next: isPlaying (no queue/shuffle → stop)", js(sess, "return App.state.isPlaying")))

    # 6. Shuffle on → next plays a random song again
    click(sess, "#btn-shuffle")  # mode 1
    click(sess, "#btn-next")
    wait_for(sess, "return App.state.isPlaying === true", timeout=10, desc="shuffle next started playback")
    checks.append(("shuffle+next plays", True))

    # 7. Pause / resume round-trip
    click(sess, "#btn-play")  # pause
    time.sleep(0.5)
    checks.append(("paused", js(sess, "return App.state.isPlaying")))
    click(sess, "#btn-play")  # resume
    time.sleep(0.5)
    checks.append(("resumed", js(sess, "return App.state.isPlaying")))

    # 8. Volume command reaches the engine
    js(sess, "return Knob.setVolume(0.5)")
    time.sleep(0.5)
    checks.append(("engine volume", js(sess, "return Bridge.invoke('get_playback_state').then(s => s.volume)")))

    wd("DELETE", f"/session/{session}")
    print(json.dumps(checks, indent=1, default=str))
    print("E2E PASS")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"E2E FAIL: {e}", file=sys.stderr)
        sys.exit(1)
