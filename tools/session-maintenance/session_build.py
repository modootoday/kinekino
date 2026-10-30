"""User-authorized Python maintenance only; product policy/tests remain Rust.

Only successfully completed scratch directories created by this wrapper are removed.
Legacy tmp, sources, fixed builds, verification evidence and global caches are excluded.
"""
from __future__ import annotations

import argparse
import contextlib
import ctypes
from ctypes import wintypes
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import time
import tomllib
import uuid

WORKSPACE = Path(__file__).resolve().parents[2]
SESSION = "01a174b5-9ce1-7402-955d-d290cf386e52"
SCHEMA = "source-session-scratch/1"
BUILD_NAMES = {"cargo.exe", "rustc.exe", "link.exe", "lld-link.exe", "compact.exe",
               "resource-runner.exe", "player.exe", "kinekino.exe",
               "media-vad-worker.exe", "source-sdk-fixture-worker.exe"}


def long_path(path: Path) -> str:
    value = os.path.abspath(path)
    if os.name != "nt" or value.startswith("\\\\?\\"):
        return value
    if value.startswith("\\\\"):
        return "\\\\?\\UNC\\" + value[2:]
    return "\\\\?\\" + value


def plain(path: Path) -> None:
    """Reject symlink/junction ancestors, without resolving through them."""
    for item in (path, *path.parents):
        try:
            st = os.lstat(long_path(item))
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(st.st_mode) or getattr(st, "st_file_attributes", 0) & 0x400:
            raise RuntimeError(f"Reparse point refused: {item}")


def active_builds(names: set[str] = BUILD_NAMES) -> list[str]:
    if os.name != "nt":
        raise RuntimeError("Production cleanup is Windows-only")

    class Entry(ctypes.Structure):
        _fields_ = [("size", wintypes.DWORD), ("usage", wintypes.DWORD),
                    ("pid", wintypes.DWORD), ("heap", ctypes.c_size_t),
                    ("module", wintypes.DWORD), ("threads", wintypes.DWORD),
                    ("parent", wintypes.DWORD), ("priority", wintypes.LONG),
                    ("flags", wintypes.DWORD), ("name", wintypes.WCHAR * 260)]

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.CreateToolhelp32Snapshot.argtypes = [wintypes.DWORD, wintypes.DWORD]
    kernel.CreateToolhelp32Snapshot.restype = wintypes.HANDLE
    kernel.Process32FirstW.argtypes = [wintypes.HANDLE, ctypes.POINTER(Entry)]
    kernel.Process32NextW.argtypes = [wintypes.HANDLE, ctypes.POINTER(Entry)]
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    handle = kernel.CreateToolhelp32Snapshot(2, 0)
    if handle == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    found = []
    try:
        entry = Entry()
        entry.size = ctypes.sizeof(entry)
        more = kernel.Process32FirstW(handle, ctypes.byref(entry))
        while more:
            if entry.name.lower() in names:
                found.append(f"{entry.name}:{entry.pid}")
            more = kernel.Process32NextW(handle, ctypes.byref(entry))
        if ctypes.get_last_error() not in (0, 18):
            raise ctypes.WinError(ctypes.get_last_error())
    finally:
        kernel.CloseHandle(handle)
    return found


def exclusive(path: Path):
    @contextlib.contextmanager
    def guard():
        import msvcrt
        plain(path.parent)
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("a+b") as file:
            if file.tell() == 0:
                file.write(b"0")
                file.flush()
            file.seek(0)
            msvcrt.locking(file.fileno(), msvcrt.LK_NBLCK, 1)
            try:
                yield
            finally:
                file.seek(0)
                msvcrt.locking(file.fileno(), msvcrt.LK_UNLCK, 1)
    return guard()


def write_new(path: Path, value: dict) -> None:
    with path.open("x", encoding="utf-8") as file:
        json.dump(value, file, ensure_ascii=False, indent=2)
        file.flush()
        os.fsync(file.fileno())


class Scratch:
    def __init__(self, workspace: Path):
        # Explicit local layout; fixtures can supply an independent workspace.
        self.root = workspace / "runtime/build-cache/tmp/session-managed" / SESSION
        self.logs = workspace / "runtime/verification/source375-datalab-b0-r1-logs"

    def completed_receipts(self) -> set[str]:
        """Recover partially removed markers only from our durable success receipt."""
        found = set()
        plain(self.logs)
        if not self.logs.exists():
            return found
        for path in self.logs.glob("*.maintenance.json"):
            plain(path)
            if path.stat().st_size > 1024 * 1024:
                continue
            data = json.loads(path.read_text(encoding="utf-8"))
            run = Path(data.get("scratch", ""))
            if (data.get("schema") == "source-build-maintenance/1"
                    and data.get("exit_code") == 0 and data.get("live_after") == []
                    and run.parent == self.root
                    and re.fullmatch(r"run-[0-9a-f]{32}", run.name)):
                found.add(run.name)
        return found

    def create(self) -> Path:
        plain(self.root)
        self.root.mkdir(parents=True, exist_ok=True)
        run = self.root / f"run-{uuid.uuid4().hex}"
        run.mkdir()
        write_new(run / "owner.json", {"schema": SCHEMA, "session": SESSION,
                                      "run": run.name, "created": time.time()})
        return run

    def candidates(self) -> list[Path]:
        plain(self.root)
        if not self.root.exists():
            return []
        result = []
        receipts = self.completed_receipts()
        for run in self.root.iterdir():
            if not re.fullmatch(r"run-[0-9a-f]{32}", run.name):
                continue
            plain(run)
            if not run.is_dir():
                continue
            owner = run / "owner.json"
            done = run / "completed.json"
            if not owner.is_file() or not done.is_file():
                if run.name in receipts:
                    result.append(run)
                continue  # Failed/unknown work without a durable success is retained.
            plain(owner)
            plain(done)
            if owner.stat().st_size > 4096 or done.stat().st_size > 4096:
                continue
            claim = json.loads(owner.read_text(encoding="utf-8"))
            completion = json.loads(done.read_text(encoding="utf-8"))
            if (claim.get("schema") == SCHEMA and claim.get("session") == SESSION
                    and claim.get("run") == run.name
                    and completion.get("run") == run.name
                    and completion.get("exit_code") == 0):
                result.append(run)
        return result

    def clean(self, apply: bool = False) -> list[str]:
        candidates = self.candidates()
        # Scan every selected tree before deleting anything; reject nested junctions.
        for run in candidates:
            def fail(error):
                raise error
            for directory, dirs, files in os.walk(long_path(run), followlinks=False, onerror=fail):
                for name in dirs + files:
                    plain(Path(directory) / name)
        if apply:
            for run in candidates:
                # This namespace only contains owned, completed scratch. Never use
                # this method for legacy tmp, incremental, deps or evidence paths.
                shutil.rmtree(long_path(run))
        return [str(path) for path in candidates]


def leftover_inventory(deps: Path) -> set[str]:
    plain(deps)
    if not deps.exists():
        return set()
    return {item.name for item in deps.iterdir() if item.is_file()
            and re.search(r"(?:\.rcgu\.o|\.tmp|\.partial)$", item.name)}


def clean_new_leftovers(deps: Path, before: set[str]) -> list[str]:
    """Only newly created loose intermediates of this successful invocation."""
    names = leftover_inventory(deps) - before
    paths = [deps / name for name in sorted(names)]
    for path in paths:
        plain(path)
    for path in paths:
        path.unlink()
    return [str(path) for path in paths]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--inspect", action="store_true", help="Read-only scratch inventory")
    parser.add_argument("--log-stem")
    parser.add_argument("--program", default="cargo")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    options = parser.parse_args()
    scratch = Scratch(WORKSPACE)
    if options.inspect:
        print(json.dumps({"eligible_completed_scratch": scratch.clean(False),
                          "free_bytes": shutil.disk_usage(WORKSPACE).free}))
        return 0
    if not options.log_stem or not re.fullmatch(r"[0-9]{4}-[a-z0-9-]+", options.log_stem):
        parser.error("Unique --log-stem is required")
    args = options.command[1:] if options.command[:1] == ["--"] else options.command
    logs = WORKSPACE / "runtime/verification/source375-datalab-b0-r1-logs"
    stem = logs / options.log_stem
    receipts = [Path(str(stem) + suffix) for suffix in
                (".stdout", ".stderr", ".receipt.json", ".maintenance.json", ".cleanup.json")]
    if any(path.exists() for path in receipts):
        raise RuntimeError("Refusing to overwrite an existing log/receipt")
    with (WORKSPACE / "rust-toolchain.toml").open("rb") as pin:
        toolchain_name = tomllib.load(pin)["toolchain"]["channel"]
    cargo_path = Path(subprocess.check_output(
        ["rustup", "which", "--toolchain", toolchain_name, "cargo"], text=True).strip())
    toolchain = cargo_path.parent
    program = cargo_path if options.program == "cargo" else Path(options.program)
    plain(program)
    if not program.is_file():
        raise RuntimeError(f"Missing program: {program}")
    if options.program != "cargo" and not program.absolute().is_relative_to(WORKSPACE / "runtime"):
        raise RuntimeError("Only explicit project runtime programs are allowed")
    target = WORKSPACE / "runtime/build-cache/native"
    if any(arg.startswith("--target-dir") for arg in args):
        raise RuntimeError("Target override refused; wrapper fixes the native context")
    for index, arg in enumerate(args):
        if arg in ("-j", "--jobs") and (index + 1 == len(args) or args[index + 1] != "1"):
            raise RuntimeError("Only jobs=1 is admitted")
        if arg.startswith("--jobs=") and arg != "--jobs=1":
            raise RuntimeError("Only jobs=1 is admitted")
        if re.fullmatch(r"-j.+", arg) and arg != "-j1":
            raise RuntimeError("Only jobs=1 is admitted")
    runner = WORKSPACE / "runtime/build-cache/resource-runner/debug/resource-runner.exe"
    plain(runner)
    with exclusive(WORKSPACE / "runtime/build-cache/session-build.lock"):
        live = active_builds()
        if live:
            raise RuntimeError(f"Heavy slot occupied: {live}")
        removed_before = scratch.clean(True)
        free_before = shutil.disk_usage(WORKSPACE).free
        if free_before < 20 * 1024**3:
            raise RuntimeError(f"Insufficient headroom: {free_before} bytes; need 20 GiB")
        run = scratch.create()
        env = os.environ.copy()
        env.update({"CARGO_TARGET_DIR": str(target), "TMP": str(run), "TEMP": str(run),
                    "CARGO_INCREMENTAL": "0", "CARGO_BUILD_JOBS": "1",
                    "RUSTUP_TOOLCHAIN": toolchain_name,
                    "RUSTC": str(toolchain / "rustc.exe")})
        with program.open("rb") as executable:
            program_sha256 = hashlib.file_digest(executable, "sha256").hexdigest()
        record = {"schema": "source-build-maintenance/1", "program": str(program),
                  "program_sha256": program_sha256,
                  "args": args, "scratch": str(run), "target": str(target),
                  "incremental": False, "free_before": free_before,
                  "removed_before": removed_before, "failed_scratch_preserved": True}
        with Path(__file__).open("rb") as source:
            record["maintenance_sha256"] = hashlib.file_digest(source, "sha256").hexdigest()
        write_new(run / "command.json", record)
        loose_before = leftover_inventory(target / "debug/deps")
        code = subprocess.call([str(runner), *(str(path) for path in receipts[:3]),
                                str(program), *args],
                               cwd=WORKSPACE, env=env)
        record["exit_code"] = code
        # A success marker is created only after runner termination and idle audit.
        live_after = active_builds()
        if code == 0 and not live_after:
            write_new(run / "completed.json", {"run": run.name, "exit_code": 0})
        record["live_after"] = live_after
        record["free_after"] = shutil.disk_usage(WORKSPACE).free
        write_new(receipts[3], record)  # Durable evidence before scratch cleanup.
        if code == 0 and not live_after:
            new_loose_removed = clean_new_leftovers(target / "debug/deps", loose_before)
            removed_after = scratch.clean(True)
            cleaned = {"exit_code": code, "removed_after": removed_after,
                       "new_loose_removed": new_loose_removed,
                       "free_bytes": shutil.disk_usage(WORKSPACE).free}
            write_new(receipts[4], cleaned)
            print(json.dumps(cleaned))
        elif live_after and code == 0:
            return 70
        return code


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"Maintenance failed closed: {error}", file=sys.stderr)
        sys.exit(70)
