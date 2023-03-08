import binascii
import io
import pathlib


def open_file(path: pathlib.Path) -> io.BytesIO:
    with path.open("rb") as fp:
        return io.BytesIO(fp.read())


if __name__ == "__main__":
    buffer = open_file(pathlib.Path("example_files/Game/SAVEGAME.sav"))
    hex = binascii.hexlify(buffer.read())
    print(hex.decode("utf-8"))
