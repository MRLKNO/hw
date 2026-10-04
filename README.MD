# hw

[English](README.md) | [Русский](README_RU.md)

A tiny CLI that scaffolds a "Hello, world!" project for many programming languages. Handy when you just need to check that a toolchain works and don't feel like writing it from scratch.

It creates projects the proper way: `cargo new` for Rust, `dotnet new` for C#, `go mod init` for Go, and so on. Optionally it builds and runs the result right away.

> The program's interface is currently in Russian. Localization is planned.

## Usage

```
hw <language> [name] [flags]
```

| Flag            | Description                      |
|-----------------|----------------------------------|
| `-b`, `--build` | create and build                 |
| `-r`, `--run`   | create, build and run            |
| `-l`, `--list`  | list supported languages         |
| `-h`, `--help`  | show help                        |

`name` is the project directory name. Defaults to `hello-<language>`.

### Examples

```
hw rust            # just create the project
hw cpp test -r     # create ./test, build and run it
hw zig --build     # create ./hello-zig and build it
hw -l              # list languages
```

## Supported languages

| Language   | Aliases              | Requires          |
|------------|----------------------|-------------------|
| rust       | `rs`                 | `cargo`           |
| c          |                      | `gcc`             |
| cpp        | `c++`, `cc`          | `g++`             |
| zig        |                      | `zig`             |
| go         | `golang`             | `go`              |
| kotlin     | `kt`                 | `kotlinc`, `java` |
| java       |                      | `javac`, `java`   |
| csharp     | `cs`, `c#`, `dotnet` | `dotnet`          |
| swift      |                      | `swiftc`          |
| nim        |                      | `nim`             |
| haskell    | `hs`                 | `ghc`             |
| python     | `py`                 | `python3`         |
| js         | `javascript`, `node` | `node`            |
| lua        |                      | `lua`             |
| dart       |                      | `dart`            |

You only need the toolchain for the languages you actually use.

## Installation

From source:

```
git clone <repo-url>
cd hw
cargo install --path .
```

As a `.deb` package (Debian/Ubuntu):

```
cargo install cargo-deb
cargo deb
sudo dpkg -i target/debian/hw_*.deb
```

## Adding a language

Every language is a single entry in the `LANGS` array in `src/main.rs`: files to write, optional scaffold/setup commands, and build/run commands. `{name}` in a command is replaced with the project name.

## Limitations

- Unix-like systems only for now: built binaries are launched as `./name`.

## License

[MIT](LICENSE)
