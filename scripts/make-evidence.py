#!/usr/bin/env python3
"""Build real evidence files so the forensics wrappers can be exercised.

Creates a packet capture, a SQLite database, a YARA rule set, a marker file, and
a small ext4 disk image (when the tools to build one are present). The image is
what the sleuthkit wrappers (mmls, fsstat, fls) need, and it is the part that is
easy to fake and therefore worth proving.
"""
import os
import shutil
import struct
import socket
import subprocess
import sys

OUT = sys.argv[1] if len(sys.argv) > 1 else "/tmp/mts-ev"
os.makedirs(OUT, exist_ok=True)


def pcap(path):
    """One UDP packet to port 53, enough for tcpdump to summarise."""
    gh = struct.pack("<IHHiIII", 0xA1B2C3D4, 2, 4, 0, 0, 65535, 1)
    eth = bytes.fromhex("ffffffffffff001122334455") + struct.pack(">H", 0x0800)
    payload = b"testdns1"
    udp = struct.pack(">HHHH", 40000, 53, 8 + len(payload), 0) + payload
    src = socket.inet_aton("10.0.0.5")
    dst = socket.inet_aton("10.0.0.6")
    total = 20 + len(udp)
    ip = struct.pack(">BBHHHBBH", 0x45, 0, total, 0, 0, 64, 17, 0) + src + dst
    pkt = eth + ip + udp
    ph = struct.pack("<IIII", 1, 0, len(pkt), len(pkt))
    open(path, "wb").write(gh + ph + pkt)


def run(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def make_ext4_image(path):
    """A tiny ext4 filesystem in a file, via mke2fs -d. Returns True on success."""
    if not shutil.which("mke2fs"):
        return False, "mke2fs not installed"
    src = os.path.join(OUT, "_img_src")
    shutil.rmtree(src, ignore_errors=True)
    os.makedirs(src)
    with open(os.path.join(src, "secret.txt"), "w") as f:
        f.write("MALWARE_TEST_MARKER - evidence file\n")
    with open(os.path.join(src, "readme.txt"), "w") as f:
        f.write("this file is part of a real ext4 image\n")
    # 8 MiB is the smallest ext4 mke2fs will make.
    with open(path, "wb") as f:
        f.truncate(8 * 1024 * 1024)
    r = run(["mke2fs", "-q", "-t", "ext4", "-d", src, "-F", path])
    if r.returncode != 0:
        return False, (r.stderr or r.stdout).strip()
    return True, "ext4 image built"


pcap(os.path.join(OUT, "evidence.pcap"))

db = os.path.join(OUT, "evidence.db")
if os.path.exists(db):
    os.remove(db)
run(["sqlite3", db, "CREATE TABLE users(id INTEGER PRIMARY KEY, name TEXT);"])
run(["sqlite3", db, "INSERT INTO users(name) VALUES ('alice'),('bob');"])

with open(os.path.join(OUT, "rules.yar"), "w") as f:
    f.write('rule Suspicious\n{\n  strings:\n    $a = "MALWARE_TEST_MARKER" ascii\n')
    f.write("  condition:\n    $a\n}\n")

with open(os.path.join(OUT, "target.bin"), "wb") as f:
    f.write(b"MALWARE_TEST_MARKER inside\n")

img = os.path.join(OUT, "evidence.img")
ok, msg = make_ext4_image(img)
print(f"evidence in {OUT}")
print(f"  pcap: {os.path.getsize(os.path.join(OUT, 'evidence.pcap'))} bytes")
print(f"  db:   {os.path.getsize(db)} bytes")
print(f"  img:  {os.path.getsize(img) if ok else 'not built'} ({msg})")
