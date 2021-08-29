# rinstall

[![builds.sr.ht status](https://builds.sr.ht/~danyspin97/rinstall/commits/.build.yml.svg)](https://builds.sr.ht/~danyspin97/rinstall/commits/.build.yml?)
![GitHub](https://img.shields.io/github/license/danyspin97/rinstall)
![Liberapay giving](https://img.shields.io/liberapay/patrons/danyspin97.svg?logo=liberapay)

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
- Flexible and configurable
- Reproducible installation

## Build

To build from source run the following command:

```
$ cargo build --release
```

To install rinstall for the current user:

```
$ ./target/release/rinstall -y
```

## Usage

If the project has an `install.yml` file present, either in the root directory or in the
`.package` directory, it supports installation via **rinstall**.

Run rinstall as your user to see the changes that will be done to the filesystem:

```
$ rinstall
```

After having reviewd the changes, add `-y` or `--yes` to perform an user installation:

```
$ rinstall -y
```

The same apply for performing a system-wide installation. Run **rinstall** as root and it
automatically switch to system-wide mode. To list the changes made to the filesystem,
run **rinstall without any arguments:

```
# rinstall
```

To accept the changes, run again the command and append `-y` or `--yes`:

```
# rinstall -y
```

You can also review the changes of a system-wide installation while running **rinstall** as
a non-privileged user:

```
$ rinstall --system
```


## Configuration

The installation directories chosen by rinstall can be configured by adding and tweaking the
file `rinstall.yml` under the `sysconfdir`. By default, `/etc/rinstall.yml` and
`$HOME/.config/rinstall.yml` will be used respectively for the root user and the non-root user.

The root configuration should already be installed by the rinstall package of your distribution and
it can also be found in the `config/root/` directory of this repository; the non-root user
configuration can be found in the `config/user/` directory. All the placeholders will be replaced at runtime by **rinstall**.

Additionally, a different configuration file can be passed by using the `--config` (or `-c`)
command line argument. All the values can also be overridden when invoking rinstall by using
the respective command line arguments.

The configuration is a YAML file that can contain the following keys. If any of them is missing,
a default value will be used instead.

- `bindir`
- `libdir`
- `datarootdir`
- `datadir`
- `sysconfdir`
- `localstatedir`
- `runstatedir`

In addition, the system-wide configuration can contain the following keys:

- `prefix`
- `exec_prefix`
- `includedir`
- `docdir`
- `mandir`

Please refer to the [Directory Variables] for their usage.

If any key is missing, 

### Placeholders in configuration

#### Root user configuration

In the configuration you may want to set a value based on another directory set prior. For example
you may want `bindir` to be a directory `bin` relative to the `exec_prefix` directory. **rinstall**
supports placeholders in the configuration to allow this:

```
exec_prefix: /usr/local
bindir: @exec_prefix@/bin
```

The root user configuration allows for the following placeholders:

- `@prefix@`, supported by all values
- `@exec_prefix@`, supported in `bindir` and `libdir`
- `@localstatedir@`, supported in `runstatedir`
- `@datarootdir@`, supported in `docdir` and `mandir`

#### Non-root user configuration

Non-root user configuration relies on XDG Directories, so it allows placeholders that refer to
these values. The placeholders will be replaced by the environment variable and, if it is not set,
it will fallback on a default value:

```
datadir: @XDG_DATA_HOME@
sysconfdir: @XDG_CONFIG_HOME@
```

The non-root user configuratione supports for the following placeholders:

- `@XDG_DATA_HOME@`, supported in `datarootdir` and `datadir`
- `@XDG_CONFIG_HOME@`, supported in `sysconfdir`
- `@XDG_STATE_HOME@`, supported in `localstatedir`
- `@XDG_RUNTIME_DIR@`, supported in `runstatedir`

## Writing `install.yml`

To support rinstall, place an `install.yml` file into the root of your project. This file
shall contains at least the name and the version of the program to install and its type. Then it
allows a list of entries differentiated by their type.

Example file:

```yaml
name: rinstall
version: 0.1
type: rust
exe:
  - rinstall
docs:
  - LICENSE.md
  - README.md
```

### Entries
Each entry list a file to install and it shall either be a string or a struct containing the
following data:

- `src`: the source, containing the location to the file that will be installed. Unless noted,
  it shall always be relative to the project directory.
- `dst`: the destination (_optional_), containing the directory or file where that this entry
  should be installed to. It shall always be relative, the corresponding system directory will
  be appended based on the type of entry; e.g. for `exe` entries, the destination part will be
  appended to `bindir`. To mark the destination as a directory, add a leading path separator `/`.
- `tmpl`: enable templating for the current entry; refer to **templating** for more information.

When the entry is only a string, it shall contains the source and follows the same rules as `src`.

Example entry defined by a struct:

```yaml
src: myprog.sh
dst: myprog
```

Example entry where destination is a directory:
```yaml
src: myprog
dst: internaldir/
```

### Type

The type part can either be `rust` or `custom`.

#### `rust`

Use `rust` type when the project is built using `cargo`. By doing so the target directory
(fetched using `cargo metadata`) will be used as root for executables and libraries.
I.e. you don't need to use `target/release/myexe` when listing executables, but just `myexe`.

### `custom`

Use `custom` for all the other projects. All the directories will be relative to the root of
the project.

### Valid keys

**rinstall** allows for the following keys:

#### `exe`

For the executables; they will be installed in `bindir` (which defaults to
`/usr/local/bin`)

#### `libs`

For the libraries; they will be installed in `libdir` (which defaults to `/usr/local/lib`)

#### `man`

For the man pages; they will be installed under the correct folder in `mandir`
(which defaults to `/usr/local/share/man`)

#### `data`

For architecture independent files; they will be installed in `datarootdir` (which
defaults to `/usr/local/share`)

#### `docs`

For documentation and examples; they will be installed in folder
`doc/<pkg-name>-<pkg-version>` under folder `datarootdir` (which defaults to
`/usr/local/share/doc/<pkg-name>-<pkg-version>`)

#### `config`

For configuration files; they will be installed in `sysconfdir` (which defaults to
`/usr/local/etc`)

#### `desktop-files`

For `.desktop` files; they will be installed in folder
`applications` under `datarootdir` (which defaults to `/usr/local/share/applications`)

#### `appdata`

For appdata files; they will be installed in folder
`appdata` under `datarootdir` (which defaults to `/usr/local/share/appdata`)

#### `completions`

For completions files; they will be installed in the respective shell completions
directory, under `datarootdir`:
- `bash-completion/completions` for *bash*
- `fish/vendor_completions.d` for *fish*
- `zsh/site-functions` for *zsh*

Example:

```yaml
completions:
  bash:
    - cat.bash
    - cp.bash
  fish:
    - cat.fish
    - cp.fish
  zsh:
    - _cat
    - _cp
```

### Templating

Sometimes it might be required to refer to some installed file or some location. However,
these locations are only known when installing, so they can't be hard-coded into
the file itself. **rinstall** allows to replace some placeholders with the actual directories.

To enable templating for a file, add `tmpl: true` to an entry:

```
docs:
  - src: my-doc.md
    tmpl: true
```

`my-doc.md` file will contains one of the placeholders specified below and they will be replaced
automatically by rinstall. For example if it contains the following contents:

```
This project has used @prefix@ as its prefix and @bindir@ as its bindir.
```

Then we invoke rinstall like this:

```
# rinstall -y --prefix /usr --bindir "@prefix@/bin"
```

The documentation file `my-doc.md` installed will look like the following:

```
This project has used /usr as its prefix and /usr/bin as its bindir.
```

#### Allowed placeholders

The following placeholders will be replaced with their respective value when templating is
enabled for an entry:

- `@prefix@`
- `@exec_prefix@`
- `@bindir@`
- `@datarootdir@`
- `@datadir@`
- `@sysconfdir@`
- `@localstatedir@`
- `@runstatedir@`
- `@includedir@`
- `@docdir@`
- `@mandir@`

## License

**rinstall** is licensed under the GPL-3+ license.

