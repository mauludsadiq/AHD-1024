import json
from pathlib import Path
from ahd1024 import aha_hash, Domain

spec = json.loads(Path("spec/test-vectors/hash-and-xof-prefreeze.json").read_text())

cases = [
    ("empty", b""),
    ("a", b"a"),
    ("abc", b"abc"),
    ("alphabet52", b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"),
    ("zero126", bytes([0]) * 126),
    ("zero128", bytes([0]) * 128),
    ("ff128", bytes([0xff]) * 128),
]

ok = True

for name, msg in cases:
    got_hash = aha_hash(msg, Domain.HASH, 32).hex()
    exp_hash = spec["HASH"][name]
    print(f"HASH {name}: {'OK' if got_hash == exp_hash else 'FAIL'}")
    if got_hash != exp_hash:
        print("  expected:", exp_hash)
        print("  got     :", got_hash)
        ok = False

    got_xof = aha_hash(msg, Domain.XOF, 64).hex()
    exp_xof = spec["XOF64"][name]
    print(f"XOF64 {name}: {'OK' if got_xof == exp_xof else 'FAIL'}")
    if got_xof != exp_xof:
        print("  expected:", exp_xof)
        print("  got     :", got_xof)
        ok = False

print("ALL_OK" if ok else "MISMATCH")
raise SystemExit(0 if ok else 1)
