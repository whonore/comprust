import sys
from itertools import zip_longest
from typing import Iterable, Iterator, Sequence, Tuple, TypeVar, Union

T = TypeVar("T")
byte = int


def chunks(xs: Iterable[T], n: int, default: T) -> Iterator[Tuple[T, ...]]:
    return zip_longest(*([iter(xs)] * n), fillvalue=default)


class Compressor:
    def encode(self, data: str) -> bytes:
        return data.encode()

    def decode(self, data: bytes) -> str:
        return data.decode()

    def test(self) -> None:
        tests = ["", "abc", "aaabccd", "aab0bb0012", "λaé", "a" * 1000]
        for test in tests:
            roundtrip = cmp.decode(cmp.encode(test))
            assert roundtrip == test, f"{roundtrip} != {test}"


class Run:
    def __init__(self, char: byte, length: int):
        self.char = char
        self.length = length

    def encode(self) -> bytes:
        return self.length.to_bytes(4, byteorder="big") + bytes((self.char,))

    @classmethod
    def decode(cls, data: bytes) -> "Run":
        len, char = data[:4], data[4]
        return cls(char, int.from_bytes(len, byteorder="big"))

    def to_bytes(self) -> bytes:
        return bytes((self.char,) * self.length)


class RLE(Compressor):
    def encode(self, data: str) -> bytes:
        return b"".join(r.encode() for r in self.runs(data.encode()))

    def decode(self, data: bytes) -> str:
        return b"".join(
            Run.decode(bytes(bs)).to_bytes() for bs in chunks(data, 5, 0)
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


if __name__ == "__main__":
    cmp = RLE() if "--rle" in sys.argv else RLE()
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
