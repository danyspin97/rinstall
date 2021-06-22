# rinstall

rinstall is an helper tool that installs software and additional data into the system.
Many programs often include man pages, documentation, config files and there is no standard
way to install them except for using Makefiles. However, Makefiles are notoriously complicated to
setup; it is especially hard to follow the [Directory Variables] from the _GNU Coding
Standard_ ([link][Makefiles Best Practices]).

[Directory Variables]: https://www.gnu.org/prep/standards/html_node/Directory-Variables.html
[Makefiles Best Practices]: https://danyspin97.org/blog/makefiles-best-practices/

rinstall read a declarative YAML file (`install.yml`) containing the list of the files to install.
It then installs the program either system-wide or for the current user (following the
[XDG BaseDirectories]). It reads the default configuration for the system from `/etc/rinstall.yml`
or `.config/rinstall.yml`, using a default one otherwise.

[XDG BaseDirectories]: https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html

## Features

- Shift the install phase from packagers to developers
- Fast
- Flexible and configurable
- Reproducible installation

## Build

To build from source run the following command:

```
$ cargo build --release
```

To install rinstall for the current user:

```
$ ./target/release/rinstall --user
```

## Usage

In a project with the `install.yml` file present, run the following command as root to perform a
system-wide installation:

```
# rinstall
```

To perform an user installation:

```
$ rinstall --user
```

## Configuration

The configuration is a YAML file that can contains the following keys:

```
prefix
exec_prefix
bindir
libdir
datarootdir
datadir
sysconfdir
localstatedir
runstatedir
includedir
docdir
mandir
```

Please refer to the [Directory Variables] for their usage.

`rinstall_user.yml` and `rinstall_root.yml` represent respectively the default user and default
root configurations. The placeholders will be replaced at runtime by **rinstall**.

The user can also override the configuration by using command line arguments.

## Writing `install.yml`

To support rinstall, place an `install.yml` file into the root of your project. This file
shall contains at least the name and the version of the program to install and its type. Then it
allows a list of files to install differentiated by their type. Each file to install shall contains
the source (`src`) part and optionally a destination (`dst`) part.

Example file:

```yaml
name: rinstall
version: 0.1
program_type: rust
exe:
  - src: rinstall
- docs:
  - src: LICENSE.md
  - src: README.md
```

The type part can either be `rust` or `custom`. In the former, the executable will be searched
inside the folder `target/release` of the root directory.

### Valid keys

**rinstall** allows for the following keys:

- `exe`, for the executables; they will be installed in `bindir` (which defaults to
  `/usr/local/bin`)
- `libs`, for the libraries; they will be installed in `libdir` (which defaults to `/usr/local/lib`)
- `man`, for the man pages; they will be installed under the correct folder in `mandir`
  (which defaults to `/usr/local/share/man`)
- `data`, for architecture independent files; they will be installed in `datarootdir` (which
  defaults to `/usr/local/share`)
- `docs`, for documentation and examples; they will be installed in folder
  `doc/<pkg-name>-<pkg-version>` under folder `datarootdir` (which defaults to
  `/usr/local/share/doc/<pkg-name>-<pkg-version>`)
- `config`, for configuration files; they will be installed in `sysconfdir` (which defaults to
  `/usr/local/etc`)
- `desktop-files`, for `.desktop` files; they will be installed in folder
  `applications` under `datarootdir` (which defaults to `/usr/local/share/applications`)
- `appdata`, for appdata files; they will be installed in folder
  `appdata` under `datarootdir` (which defaults to `/usr/local/share/appdata`)
- `completions`, for completions files; they will be installed in the respective shell completions
  directory, under `datarootdir`:
  + `bash-completion/completions` for *bash*
  + `fish/vendor_completions.d` for *fish*
  + `zsh/site-functions` for *zsh*

## TODO

- Add `--reversible` (reverse the installation)
- Add `--exclude` flag

## License

**rinstall** is licensed under the GPL-3+ license.

