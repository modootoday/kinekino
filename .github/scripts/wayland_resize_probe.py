#!/usr/bin/env python3
"""Wayland resize probe for the player window (M6-5b), stdlib only.

Runs inside a headless sway session. It finds the window of process --pid, then measures
how long the client takes to show a frame at a new size after each resize:
  1. compositor resizes (`resize set`), and
  2. an edge drag with a virtual pointer (wlr-virtual-pointer), which the player hands to
     the compositor through BeginResize (xdg_toplevel.resize).
"Followed" means sway reports the client's committed surface geometry at the new size;
a grim capture of the window is saved after each step and checked for unpainted pixels
(the output background is a solid colour no player pixel uses).
Writes results.json and PNGs under --out.
"""
import argparse, json, os, socket, struct, subprocess, sys, time

MAGIC = b"i3-ipc"


class Sway:
    def __init__(self, path):
        self.s = socket.socket(socket.AF_UNIX)
        self.s.connect(path)

    def _recv(self, n):
        buf = b""
        while len(buf) < n:
            chunk = self.s.recv(n - len(buf))
            if not chunk:
                raise RuntimeError("sway ipc closed")
            buf += chunk
        return buf

    def call(self, kind, payload=b""):
        self.s.sendall(MAGIC + struct.pack("<II", len(payload), kind) + payload)
        head = self._recv(14)
        size, _ = struct.unpack("<II", head[6:])
        return json.loads(self._recv(size))

    def command(self, text):
        return self.call(0, text.encode())

    def tree(self):
        return self.call(4)


def find_view(node, pid):
    if node.get("pid") == pid:
        return node
    for child in node.get("nodes", []) + node.get("floating_nodes", []):
        found = find_view(child, pid)
        if found:
            return found
    return None


class VirtualPointer:
    """Raw Wayland wire client: zwlr_virtual_pointer_manager_v1 -> virtual pointer."""

    def __init__(self, runtime, display):
        self.s = socket.socket(socket.AF_UNIX)
        self.s.connect(os.path.join(runtime, display))
        self.next_id = 2
        self.buf = b""
        registry = self.new_id()
        self.send(1, 1, struct.pack("<I", registry))  # wl_display.get_registry
        globals_ = self.roundtrip(registry)
        name = next((n for n, (iface, _) in globals_.items()
                     if iface == "zwlr_virtual_pointer_manager_v1"), None)
        if name is None:
            raise RuntimeError("compositor has no zwlr_virtual_pointer_manager_v1")
        self.manager = self.new_id()
        iface = b"zwlr_virtual_pointer_manager_v1\0"
        iface += b"\0" * (-len(iface) % 4)
        self.send(registry, 0, struct.pack("<II", name, len(b"zwlr_virtual_pointer_manager_v1") + 1)
                  + iface + struct.pack("<II", 1, self.manager))
        self.pointer = self.new_id()
        self.send(self.manager, 0, struct.pack("<II", 0, self.pointer))  # seat=null
        self.roundtrip(None)

    def new_id(self):
        self.next_id += 1
        return self.next_id - 1

    def send(self, obj, opcode, args=b""):
        self.s.sendall(struct.pack("<II", obj, ((8 + len(args)) << 16) | opcode) + args)

    def roundtrip(self, registry):
        callback = self.new_id()
        self.send(1, 0, struct.pack("<I", callback))  # wl_display.sync
        globals_ = {}
        while True:
            while len(self.buf) < 8:
                self.buf += self.s.recv(65536)
            obj, word = struct.unpack("<II", self.buf[:8])
            size, opcode = word >> 16, word & 0xFFFF
            while len(self.buf) < size:
                self.buf += self.s.recv(65536)
            body, self.buf = self.buf[8:size], self.buf[size:]
            if obj == registry and opcode == 0:
                name, slen = struct.unpack("<II", body[:8])
                iface = body[8:8 + slen - 1].decode()
                globals_[name] = (iface, body)
            elif obj == 1 and opcode == 0:  # wl_display.error
                raise RuntimeError(f"wayland error {body!r}")
            elif obj == callback and opcode == 0:
                return globals_

    def ms(self):
        return int(time.monotonic() * 1000) & 0xFFFFFFFF

    def move_to(self, x, y, extent):
        self.send(self.pointer, 1, struct.pack("<IIIII", self.ms(), int(x), int(y), *extent))
        self.send(self.pointer, 4, struct.pack("<I", self.ms()))  # frame

    def button(self, pressed):
        self.send(self.pointer, 2, struct.pack("<III", self.ms(), 0x110, 1 if pressed else 0))
        self.send(self.pointer, 4, struct.pack("<I", self.ms()))
        self.roundtrip(None)


def wait_for(sway, pid, check, timeout):
    start = time.monotonic()
    polls = 0
    while time.monotonic() - start < timeout:
        view = find_view(sway.tree(), pid)
        polls += 1
        if view and check(view):
            return (time.monotonic() - start) * 1000.0, view, polls
        time.sleep(0.001)
    return None, find_view(sway.tree(), pid), polls


def geometry(view):
    g = view["geometry"]
    return g["width"], g["height"]


def capture(out, name, view, background):
    r = view["rect"]
    path = os.path.join(out, name)
    subprocess.run(["grim", "-g", f"{r['x']},{r['y']} {r['width']}x{r['height']}", path], check=True)
    ppm = subprocess.run(["grim", "-t", "ppm", "-g", f"{r['x']},{r['y']} {r['width']}x{r['height']}", "-"],
                         check=True, capture_output=True).stdout
    # P6\n<w> <h>\n255\n<rgb...>
    parts = ppm.split(b"\n", 3)
    data = parts[3]
    total = len(data) // 3
    unpainted = sum(1 for i in range(0, len(data), 3) if data[i:i + 3] == background)
    return {"png": name, "rect": [r["x"], r["y"], r["width"], r["height"]],
            "unpainted_fraction": round(unpainted / max(total, 1), 5)}


def stats(values):
    v = sorted(values)
    if not v:
        return None
    pick = lambda q: v[min(len(v) - 1, int(round(q * (len(v) - 1))))]
    return {"n": len(v), "min": round(v[0], 1), "p50": round(pick(0.5), 1),
            "p95": round(pick(0.95), 1), "max": round(v[-1], 1)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pid", type=int, required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--output-size", default="1600x1000")
    ap.add_argument("--rounds", type=int, default=5)
    ap.add_argument("--timeout", type=float, default=5.0)
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    extent = tuple(int(x) for x in a.output_size.split("x"))
    background = bytes([0xFF, 0x00, 0xFF])
    sway = Sway(os.environ["SWAYSOCK"])
    result = {"pid": a.pid, "output": a.output_size, "ok": False, "errors": []}

    took, view, _ = wait_for(sway, a.pid, lambda v: v["geometry"]["width"] > 0, 60)
    if took is None:
        result["errors"].append("player window never mapped")
        json.dump(result, open(os.path.join(a.out, "results.json"), "w"), indent=1)
        return 1
    result["window"] = {"app_id": view.get("app_id"), "shell": view.get("shell"),
                        "name": view.get("name"), "mapped_ms": round(took, 1)}
    con = view["id"]
    sway.command(f"[con_id={con}] floating enable, move position 40 40, resize set 960 600")
    _, view, _ = wait_for(sway, a.pid, lambda v: geometry(v) == (960, 600), a.timeout)
    time.sleep(1.0)
    result["captures"] = [capture(a.out, "start.png", find_view(sway.tree(), a.pid), background)]

    # 1. Compositor resize: configure -> client commit at the new size.
    sizes = [(1280, 800), (720, 450), (1100, 700), (640, 400), (1440, 900), (960, 600)]
    configure = []
    for round_ in range(a.rounds):
        for w, h in sizes:
            sway.command(f"[con_id={con}] resize set {w} px {h} px")
            took, view, polls = wait_for(sway, a.pid, lambda v, w=w, h=h: geometry(v) == (w, h), a.timeout)
            row = {"round": round_, "target": [w, h], "followed_ms": took and round(took, 1),
                   "got": list(geometry(view)) if view else None, "polls": polls}
            configure.append(row)
            if took is None:
                result["errors"].append(f"configure {w}x{h}: no commit at that size")
            if round_ == 0:
                time.sleep(0.4)
                result["captures"].append(capture(a.out, f"configure-{w}x{h}.png", view, background))
    result["configure"] = {"rows": configure,
                           "stats": stats([r["followed_ms"] for r in configure if r["followed_ms"] is not None])}

    # 2. Edge drag: virtual pointer presses on the right edge band and drags.
    drag = {"steps": [], "errors": []}
    try:
        vp = VirtualPointer(os.environ["XDG_RUNTIME_DIR"], os.environ["WAYLAND_DISPLAY"])
        sway.command(f"[con_id={con}] move position 40 40, resize set 960 600")
        wait_for(sway, a.pid, lambda v: geometry(v) == (960, 600), a.timeout)
        time.sleep(0.5)
        view = find_view(sway.tree(), a.pid)
        r = view["rect"]
        x, y = r["x"] + r["width"] - 3, r["y"] + r["height"] // 2
        vp.move_to(x, y, extent)
        time.sleep(0.3)
        vp.button(True)
        time.sleep(0.3)
        start_w = geometry(view)[0]
        for step in range(1, 9):
            nx = x + 40 * step
            vp.move_to(nx, y, extent)
            want = start_w + 40 * step
            took, view, polls = wait_for(sway, a.pid, lambda v, want=want: geometry(v)[0] == want, a.timeout)
            drag["steps"].append({"pointer_x": nx, "want_width": want, "followed_ms": took and round(took, 1),
                                  "got": list(geometry(view)) if view else None})
        vp.button(False)
        time.sleep(0.5)
        view = find_view(sway.tree(), a.pid)
        drag["final"] = list(geometry(view))
        drag["resized"] = drag["final"][0] > start_w
        drag["stats"] = stats([s["followed_ms"] for s in drag["steps"] if s["followed_ms"] is not None])
        result["captures"].append(capture(a.out, "after-drag.png", view, background))
        if not drag["resized"]:
            drag["errors"].append("edge drag did not resize the window")
    except Exception as e:  # recorded, not hidden
        drag["errors"].append(repr(e))
    result["drag"] = drag
    result["errors"] += drag["errors"]
    painted = all(c["unpainted_fraction"] < 0.01 for c in result["captures"])
    result["painted"] = painted
    if not painted:
        result["errors"].append("a capture shows unpainted window area")
    result["ok"] = not result["errors"]
    json.dump(result, open(os.path.join(a.out, "results.json"), "w"), indent=1)
    print(json.dumps({k: result.get(k) for k in ("ok", "window", "errors")}))
    print(json.dumps({"configure": result["configure"]["stats"], "drag": drag.get("stats"),
                      "drag_final": drag.get("final")}))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
