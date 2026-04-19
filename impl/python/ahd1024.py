from __future__ import annotations

from dataclasses import dataclass
from enum import IntEnum
from hashlib import shake_256
from typing import List

RATE_BITS = 1024
RATE_BYTES = RATE_BITS // 8
ROUNDS = 24
SEED = b"AHA-D-256-ROUND-CONSTANTS-v0.1"

ROT = [
    [0, 7, 19, 41, 53],
    [11, 29, 43, 3, 31],
    [37, 59, 5, 17, 47],
    [23, 13, 61, 27, 9],
    [45, 21, 39, 49, 55],
]

MASK64 = (1 << 64) - 1


class Domain(IntEnum):
    HASH = 0x01
    XOF = 0x02
    TREE_LEAF = 0x03
    TREE_PARENT = 0x04
    MAC_KEYED = 0x05
    TRANSCRIPT = 0x06
    ARTIFACT = 0x07
    ROUND_TRACE = 0x08


@dataclass(frozen=True)
class Constants:
    k0: List[int]
    k1: List[int]
    k2: List[int]


State = List[List[int]]


def rotl64(x: int, n: int) -> int:
    n &= 63
    return ((x << n) | (x >> (64 - n))) & MASK64


def derive_constants() -> Constants:
    material = shake_256(SEED).digest(3 * ROUNDS * 8)
    words = [int.from_bytes(material[i:i + 8], "little") for i in range(0, len(material), 8)]
    k0 = [words[3 * t] for t in range(ROUNDS)]
    k1 = [words[3 * t + 1] for t in range(ROUNDS)]
    k2 = [words[3 * t + 2] for t in range(ROUNDS)]
    return Constants(k0=k0, k1=k1, k2=k2)


def blank_state() -> State:
    return [[0 for _ in range(5)] for _ in range(5)]


def theta(s: State) -> State:
    c = [s[x][0] ^ s[x][1] ^ s[x][2] ^ s[x][3] ^ s[x][4] for x in range(5)]
    d = [
        rotl64(c[(x + 4) % 5], 1) ^ rotl64(c[(x + 1) % 5], 11) ^ rotl64(c[(x + 2) % 5], 27)
        for x in range(5)
    ]
    out = blank_state()
    for x in range(5):
        for y in range(5):
            out[x][y] = s[x][y] ^ d[x]
    return out


def pi_stage(s: State) -> State:
    out = blank_state()
    for x in range(5):
        for y in range(5):
            out[x][y] = s[(2 * x + 3 * y) % 5][(x + 2 * y) % 5]
    return out


def rho(s: State) -> State:
    out = blank_state()
    for x in range(5):
        for y in range(5):
            out[x][y] = rotl64(s[x][y], ROT[x][y])
    return out


def chi_star(s: State) -> State:
    out = blank_state()
    for y in range(5):
        a = [s[0][y], s[1][y], s[2][y], s[3][y], s[4][y]]
        for i in range(5):
            out[i][y] = (
                a[i]
                ^ ((~a[(i + 1) % 5] & MASK64) & a[(i + 2) % 5])
                ^ (rotl64(a[(i + 3) % 5], 1) & rotl64(a[(i + 4) % 5], 3))
            ) & MASK64
    return out


def iota(s: State, t: int, constants: Constants) -> State:
    out = [row[:] for row in s]
    out[0][0] ^= constants.k0[t]
    out[1][2] ^= constants.k1[t]
    out[4][4] ^= constants.k2[t]
    out[0][0] &= MASK64
    out[1][2] &= MASK64
    out[4][4] &= MASK64
    return out


def permute(s: State, rounds: int, constants: Constants) -> State:
    state = [row[:] for row in s]
    for t in range(rounds):
        state = theta(state)
        state = pi_stage(state)
        state = rho(state)
        state = chi_star(state)
        state = iota(state, t, constants)
    return state


def pad_v02(message: bytes, domain: Domain) -> bytes:
    out = bytearray(message)
    out.append(int(domain))
    out.append(0x01)
    while len(out) % RATE_BYTES != RATE_BYTES - 1:
        out.append(0x00)
    out.append(0x80)
    return bytes(out)


def absorb_blocks(padded: bytes, rounds: int, constants: Constants) -> State:
    s = blank_state()
    for block_start in range(0, len(padded), RATE_BYTES):
        block = padded[block_start:block_start + RATE_BYTES]
        for i in range(16):
            lane = int.from_bytes(block[8 * i:8 * i + 8], "little")
            x = i % 5
            y = i // 5
            s[x][y] ^= lane
            s[x][y] &= MASK64
        s = permute(s, rounds, constants)
    return s


def squeeze_bytes(s: State, out_len: int, rounds: int, constants: Constants) -> bytes:
    state = [row[:] for row in s]
    out = bytearray()
    while len(out) < out_len:
        for i in range(16):
            x = i % 5
            y = i // 5
            out.extend(state[x][y].to_bytes(8, "little"))
            if len(out) >= out_len:
                return bytes(out[:out_len])
        state = permute(state, rounds, constants)
    return bytes(out[:out_len])


def aha_hash(message: bytes, domain: Domain, out_len: int, rounds: int = ROUNDS) -> bytes:
    constants = derive_constants()
    padded = pad_v02(message, domain)
    s = absorb_blocks(padded, rounds, constants)
    return squeeze_bytes(s, out_len, rounds, constants)


if __name__ == "__main__":
    print(aha_hash(b"", Domain.HASH, 32).hex())
