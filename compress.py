import math
import os
import sys
from itertools import zip_longest
from typing import (
    Iterable,
    Iterator,
    List,
    Mapping,
    Sequence,
    Tuple,
    TypeVar,
    Union,
)
from typing_extensions import Final

T = TypeVar("T")
byte = int

DEBUG = bool(int(os.environ.get("DEBUG", False)))


def debug(*args, **kwargs):
    if DEBUG:
        print(*args, **kwargs, file=sys.stderr)


def chunks(xs: Iterable[T], n: int, default: T) -> Iterator[Tuple[T, ...]]:
    return zip_longest(*([iter(xs)] * n), fillvalue=default)


class Compressor:
    def encode(self, data: str) -> bytes:
        return data.encode()

    def decode(self, data: bytes) -> str:
        return data.decode()

    def test(self) -> None:
        tests = ["", "abc", "aaabccd", "aab0bb0012", "λaé", "a" * 1000, "ababcaab"]
        for test in tests:
            roundtrip = self.decode(self.encode(test))
            assert roundtrip == test, f"{roundtrip} != {test}"


class Run:
    def __init__(self, char: byte, length: int):
        self.char = char
        self.length = length

    def encode(self) -> bytes:
        return self.length.to_bytes(4, "big") + self.char.to_bytes(1, "big")

    @classmethod
    def decode(cls, data: bytes) -> "Run":
        len, char = data[:4], data[4]
        return cls(char, int.from_bytes(len, byteorder="big"))

    def pack(self) -> bytes:
        return bytes((self.char,) * self.length)


class RLE(Compressor):
    def encode(self, data: str) -> bytes:
        return b"".join(r.encode() for r in self.runs(data.encode()))

    def decode(self, data: bytes) -> str:
        return b"".join(
            Run.decode(bytes(bs)).pack() for bs in chunks(data, 5, 0)
        ).decode()

    @staticmethod
    def runs(data: bytes) -> Sequence[Run]:
        if data == b"":
            return []

        runs = [Run(data[0], 0)]
        for char in data:
            prev = runs[-1]
            if prev.char == char:
                prev.length += 1
            else:
                runs.append(Run(char, 1))

        return runs


class Bits:
    def __init__(self) -> None:
        self.bits: List[Tuple[int, int]] = []

    def append(self, val: int, width: int) -> None:
        assert val < 2 ** width
        self.bits.append((val, width))

    def __str__(self) -> str:
        return "".join(f"{val:0{width}b}" for val, width in self.bits)

    def pack(self) -> bytes:
        return bytes(int("".join(bs), 2) for bs in chunks(str(self), 8, "0"))


class LZWDict:
    stop_code: Final[int] = 0

    class _LZWRevDict:
        def __init__(self, dict: Mapping[bytes, int]) -> None:
            self.dict = {code: c for c, code in dict.items()}

        def __getitem__(self, code: int) -> bytes:
            return self.dict[code]

        def __contains__(self, code: int) -> bool:
            return code in self.dict

    def __init__(self, alph: Sequence[bytes]) -> None:
        self.dict = {c: code for code, c in enumerate(alph, start=self.stop_code + 1)}
        self.reversed = self._LZWRevDict(self.dict)
        self.max_code = max(self.dict.values())

    def _nbits(self, n: int) -> int:
        return math.ceil(math.log2(n + 1))

    @property
    def nbits(self) -> int:
        return self._nbits(self.max_code)

    @property
    def nbits_next(self) -> int:
        return self._nbits(self.max_code + 1)

    def insert(self, bs: bytes) -> None:
        self.max_code += 1
        assert bs not in self.dict
        self.dict[bs] = self.max_code
        assert self.max_code not in self.reversed.dict
        self.reversed.dict[self.max_code] = bs

    def __getitem__(self, bs: bytes) -> int:
        return self.dict[bs]

    def __contains__(self, bs: bytes) -> bool:
        return bs in self.dict


class LZW(Compressor):
    alph: Final[Sequence[bytes]] = tuple(c.to_bytes(1, "big") for c in range(256))

    def encode(self, data: str) -> bytes:
        bits = LZW.bits(data.encode(), LZWDict(self.alph))
        debug(f"Out:\n{str(bits)}")
        return bits.pack()

    def decode(self, data: bytes) -> str:
        dict = LZWDict(self.alph)
        bin = "".join(f"{b:08b}" for b in data)
        debug(f"In:\n{bin}")

        out = bytearray()
        nbits = dict.nbits
        prev_bs = None
        while bin != "":
            code, bin = int(bin[:nbits], 2), bin[nbits:]
            if code == dict.stop_code:
                break

            try:
                bs = dict.reversed[code]
            except KeyError:
                assert prev_bs is not None
                assert code == dict.max_code + 1
                bs = prev_bs + bytes((prev_bs[0],))
            debug(f"Emitting {bs} ({code})", end="")
            out += bs

            if prev_bs is not None:
                prev_bs += bytes((bs[0],))
                assert prev_bs not in dict
                debug(f"\t--\tInserting {prev_bs} ({dict.max_code})", end="")
                dict.insert(prev_bs)
                nbits = dict.nbits_next

            debug()
            assert bs == dict.reversed[code]
            prev_bs = bs

        return out.decode()

    @staticmethod
    def bits(data: bytes, dict: LZWDict) -> Bits:
        word = bytearray()
        bits = Bits()
        for c in data:
            word.append(c)
            if bytes(word) not in dict:
                bits.append(dict[bytes(word[:-1])], dict.nbits)
                dict.insert(bytes(word))
                debug(
                    f"Emitting {bytes(word[:-1])} ({dict[bytes(word[:-1])]})\t--\t"
                    f"Inserting {bytes(word)} ({dict.max_code})"
                )
                word = bytearray([c])
        assert len(word) > 0 or len(data) == 0
        if word != bytearray():
            assert bytes(word) in dict
            debug(f"Emitting {bytes(word)} ({dict[bytes(word)]})")
            bits.append(dict[bytes(word)], dict.nbits)
        bits.append(dict.stop_code, dict.nbits)
        return bits


if __name__ == "__main__":
    cmp = RLE() if "--rle" in sys.argv else LZW() if "--lzw" in sys.argv else RLE()
    if "-t" in sys.argv:
        cmp.test()
    else:
        data: Union[str, bytes]
        if "-e" in sys.argv:
            data = sys.stdin.read(-1)
            sys.stdout.buffer.write(cmp.encode(data))
        elif "-d" in sys.argv:
            data = sys.stdin.buffer.read(-1)
            print(cmp.decode(data))
        else:
            data = sys.stdin.read(-1)
            enc = cmp.encode(data)
            dec = cmp.decode(enc)
            print(f"In:  {data}\nEnc: {enc!r}\nDec: {dec}")
