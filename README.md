# decayfmt

[![CI](https://github.com/aravpanwar/decayfmt/actions/workflows/ci.yml/badge.svg)](https://github.com/aravpanwar/decayfmt/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/decayfmt.svg)](https://crates.io/crates/decayfmt)
[![PyPI](https://img.shields.io/pypi/v/decayfmt-py.svg)](https://pypi.org/project/decayfmt-py/)

_Featured in [This Week in Rust #660](https://this-week-in-rust.org/blog/2026/07/15/this-week-in-rust-660/)._

**A file format that corrupts itself a little every time you open it.** Every open
permanently damages the file on disk, by an amount baked into the filename, before it is
ever shown to you. There is no recovery from the file alone. The file is the only copy
that matters, and every read destroys a little more of it.

![The same image, encoded at four instability values and opened in step, decaying at four speeds at once](assets/decay-grid.gif)

Two file types:

- `.idcy<x>` for images (example: `photo.idcy3`)
- `.tdcy<x>` for text (example: `note.tdcy7`)

`x` is a positive integer in the filename, the instability parameter. Higher `x` means
more corruption per open.

## Watch it decay

The grid above is one image encoded at `x=1`, `x=3`, `x=8`, and `x=15`, each opened the
same number of times. Same picture, four rates of decay. To follow a single instability
value across individual opens instead, each open corrupting the file further on disk
before it is ever shown, with no way back:

The clean original:

![Original](assets/original.png)

| Instability | After 1 open | After 3 opens |
| :---: | :---: | :---: |
| `x=3` (gentle) | ![x3 after one open](assets/x3-after-1.png) | ![x3 after three opens](assets/x3-after-3.png) |
| `x=10` (severe) | ![x10 after one open](assets/x10-after-1.png) | ![x10 after three opens](assets/x10-after-3.png) |

At `x=3` the image degrades gracefully over many opens. At `x=10` it is nearly gone after
one open and pure noise after three. `x` is the dial between a slow fade and near-instant
destruction.

Text decays the same way. A sentence encoded at `x=1` (a slow burn), printed after a few
opens:

```text
original : This sentence is dying, and every time you read it you kill it a little more.
 open 1  : This sgntence is d+ingd !nd every time you re&p it P~u kiKl it a little more}
 open 3  : This sgfxFn0e is d+ingd 3D6 every tibe you re&" it P~u kiKl it a 1ittl> m1re}
 open 6  : TIbm sgf}Fn0e ts d+iqgd yD6 ev*ry tibe you re&" )t Pnu kiKB )t aC1it"l> m1^e}
 open 9  : T/Sm sgf}Fk0- ts d|iqgd HD6 e@*rV tiFe you re&" )t Pnu kiKB )tpaC1it"lYMm1^>}
 open 12 : h/Sm hgf}Nk0-'ts?K|iqgd HD6 e@`~V}t&Fe y%u re&" )2 Pnu kiKB )6UaC1it1lYMm1]b!
```

Corruption only ever swaps in printable characters, so the text rots into readable-looking
nonsense.

## What this is, and is not

decayfmt is a social contract, not a security tool. Do not treat it as encryption, DRM, or
a way to securely wipe a file. The corruption is honest and unrecoverable from the file
alone, but anyone with a backup or a hex editor can defeat it. If you want the original,
keep a backup. If you do not want anyone to recover it, do not make one.

## FAQ

A video sent a lot of people here at once, so here are the questions I keep getting.

**Does the file corrupt itself?**

No. Files are just data and cannot change on their own. decayfmt's `open` command is what
corrupts the file: it reads the bytes, damages some of them, writes them back, and then shows
you the result. A different program opening the same file would leave it untouched. The
"self-corrupting" line is shorthand for "the tool corrupts it every time you use the tool to
look at it."

**Is this DRM? Could a company use it to make you re-buy games?**

No, and it would be a terrible way to try. Anyone can copy the file before opening it and keep
the original forever, so a backup defeats the whole thing in one step. It also needs write
access to work, and a game that damaged its own files on every launch would break itself
almost immediately. If a company wanted your files gone, they would just delete them. This
gives nobody a power they did not already have.

**Is it dangerous? Is it malware?**

No. It only touches files you deliberately encode into the decayfmt format and then open with
decayfmt. It does not scan your disk or run on its own. It is a toy, not a weapon.

**Can I get a decayed file back?**

Not from the file itself. Once the corruption is written, the earlier version is gone. If you
want the original, keep a backup. That is the entire point.

**Why does it exist?**

Mostly for fun. I liked the idea of a file you could use up, like a print left in the sun.
There is no serious use case, and I have been upfront about that from the start.

## Install

### With cargo

If you have a Rust toolchain, the quickest install is the published crate:

```
cargo install decayfmt
```

### From a release

Download the binary for your platform from the
[releases page](https://github.com/aravpanwar/decayfmt/releases) and put it on your PATH.
There is no runtime dependency to install.

On macOS the binary is unsigned, so the first run may be blocked by Gatekeeper. Right-click
it and choose Open, or clear the quarantine flag with `xattr -d com.apple.quarantine decayfmt`.

### From source

Requires a Rust toolchain.

```
cargo build --release
```

The binary is produced at `target/release/decayfmt`.

## Quickstart

See it decay in your terminal, with no image or sample file needed:

```
echo "this sentence is about to start dying" > note.txt
decayfmt encode --input note.txt --output note.tdcy8
decayfmt open note.tdcy8
```

The instability `x` comes from the output name (`note.tdcy8` decays at `x=8`). Run that last
line a few more times and watch the sentence rot further on each open. The corruption is
written to disk before it prints, so there is no way back. A high `x` like 8 garbles it
fast; a low `x` like 1 is a slow burn over many opens.

On Windows PowerShell the `>` redirect writes UTF-16, which decayfmt refuses; create the
file with `Set-Content note.txt "this sentence is about to start dying"` instead. cmd.exe
and PowerShell 7 are fine with the line above.

## Usage

### Encode

Turn a source image or text file into a decayfmt file. Encoding never corrupts; the new
file is clean.

```
decayfmt encode --input photo.png --output photo.idcy3
decayfmt encode --input note.txt  --output note.tdcy7
```

Both the file type and the instability `x` come from the output name: `idcy` for images and
`tdcy` for text, followed by `x` as a positive integer (`photo.idcy3` is an image at `x=3`).
An output name that could never be opened is refused rather than written. Images are decoded
to raw RGBA; text must be valid UTF-8.

### Open

Open a decayfmt file. This corrupts it in place on disk, then displays the result.
Images open in your system's default image viewer. Text prints to the terminal, and
when there is no terminal (for example when launched from a file manager) it also
opens in your default text editor.

```
decayfmt open photo.idcy3
decayfmt open note.tdcy7
```

`x` is read from the filename, so renaming the file changes how hard the next open hits.

## Python

The same library is available from Python as the `decayfmt` package — same format,
same statistics, same errors as the CLI:

```
pip install decayfmt-py
```

```python
import decayfmt

decayfmt.encode_file("notes.txt", "notes.tdcy5")

# Each open permanently corrupts the file on disk.
for _ in range(5):
    kind, dims, data = decayfmt.decay_file("notes.tdcy5")
    print(data[16:].decode("utf-8", errors="replace"))

# Or decay bytes in memory, no files involved:
data = b"the quick brown fox jumps over the lazy dog"
for _ in range(8):
    data = decayfmt.corrupt_bytes(data, 1.0, "text")
```

The binding exposes `corrupt_bytes` (copy out), `corrupt_in_place` (zero-copy on a
`bytearray`), `encode_file`, `encode_bytes`, `decay_file` (open without display),
`parse_filename`, `read_header`, and `write_header`. The GIL is released while
corruption runs, and large payloads are corrupted in parallel across cores. Full
documentation is on [PyPI](https://pypi.org/project/decayfmt-py/) and in
[`python/README.md`](python/README.md).

## Performance

Corruption was rewritten around bulk random draws and a parallel region split
(each region seeds its own OS-entropy generator), and encoding no longer builds a
second full-size buffer. Measured on an Apple silicon Mac (10 logical cores),
64 MiB payload, release build, `cargo bench --bench corrupt` / `--bench encode`:

| Scenario | Before | After | Speedup |
| :--- | ---: | ---: | ---: |
| text corruption, x=1 | 142 MiB/s | 1,366 MiB/s | 9.6x |
| text corruption, x=10 | 86 MiB/s | 754 MiB/s | 8.8x |
| image corruption, x=1 | 187 MiB/s | 1,421 MiB/s | 7.6x |
| image corruption, x=10 | 116 MiB/s | 950 MiB/s | 8.2x |
| encode 16 MiB image | 33.2 ms | 32.3 ms | ~1x |
| encode 16 MiB text | 7.9 ms | 3.9 ms | 2.0x |

The corruption loop itself is now bounded by entropy generation and memory
bandwidth rather than per-byte RNG calls. End-to-end encode is dominated by image
decoding and disk I/O, which the rewrite does not touch, so those numbers move
little. The same distribution holds: per-byte probabilities are quantized to
1/65536 granularity, about 1.5e-5 absolute error, far below anything a filename
`x` can distinguish, and the statistical test suite verifies it.

## How the corruption works

On each open, a per-byte corruption probability is derived from `x`:

```
p = 1 - exp(-x / 10)
```

So `x = 1` corrupts roughly 9.5% of eligible bytes per open, `x = 5` roughly 39%, and
`x = 10` roughly 63%. The randomness comes from a cryptographically secure generator
seeded from operating system entropy, never from a fixed seed, so two opens of the same
state look different and the corruption sequence cannot be replayed.

- **Images:** the red, green, and blue channels are each corrupted independently with
  probability `p`. The alpha channel is never touched, so corruption shows as color
  noise rather than transparency holes.
- **Text:** each byte is replaced, with probability `p`, by a random printable ASCII
  byte. This operates on bytes, not characters, so at high `x` it can break UTF-8; the
  viewer renders what it can and substitutes the replacement character for the rest.
  Corruption substitutes bytes in place and never inserts or deletes, so the file length
  and the positions of untouched bytes are preserved: content decays but structure does
  not. The original byte length is always recoverable, and at low `x` word lengths and
  layout largely survive. Spaces are not protected; they are replaced at the same rate as
  any other byte and erode along with everything else as `x` rises.

## The contract

- Corruption is written to disk at open time, before display. A crash or kill after the
  write does not undo it. Opening always costs a corruption.
- A read-only file is refused with an error and never displayed. A free read would break
  the contract.
- The header is never changed after encoding. Only the payload decays.
- There is no state in the file: no read counter, no timestamp, no record of who opened
  it or when.
- There is no recovery mechanism of any kind.

## Limitations

- This is a social contract, not cryptography. A backup defeats it entirely.
- A determined person with a hex editor can tamper with the file.
- It is not a secure deletion tool and makes no cryptographic guarantee.
- Displaying a file writes the corrupted result to a temporary file for the system viewer.
  The most recent one persists until the next open sweeps it, or indefinitely if there is
  no next open, so a snapshot of the last-shown state stays recoverable until then.
- Two opens running at the same time can race: both read the same starting state, and the
  last write wins, so concurrent opens may cost fewer corruptions than sequential ones.
- v1 supports images and text only. No audio, video, or other binary formats.

## License

decayfmt is released under the MIT License. See [LICENSE](LICENSE).

---

[HN Discussion](https://news.ycombinator.com/item?id=49390206)
