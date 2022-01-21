# rinstall

[![builds.sr.ht status](https://builds.sr.ht/~danyspin97/rinstall/commits/.build.yml.svg)](https://builds.sr.ht/~danyspin97/rinstall/commits/.build.yml?)
![GitHub](https://img.shields.io/github/license/danyspin97/rinstall)
![Liberapay giving](https://img.shields.io/liberapay/patrons/danyspin97.svg?logo=liberapay)

rinstall is an helper tool that installs software and additional data into the system.
Many programs often include man pages, documentation, config files and there is no standard
way to install them except for using Makefiles. However, Makefiles are notoriously complicated to
setup; it is especially hard to follow the [Directory Variables] from the _GNU Coding
Standard_.).

[Directory Variables]: https://www.gnu.org/prep/standards/html_node/Directory-Variables.html

[This][Makefiles Best Practices] is rinstall rationale.

[Makefiles Best Practices]: https://danyspin97.org/blog/makefiles-best-practices/

rinstall read a declarative YAML file (`install.yml`) containing the list of the files to install.
It then installs the program either system-wide or for the current user (following the
[XDG BaseDirectories]). It reads the default configuration for the system from `/etc/rinstall.yml`
or `.config/rinstall.yml`, using a default one otherwise.

[XDG BaseDirectories]: https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html

## Features

- List the files to install and their location in a declarative way
- Ensure backward compatibility (no need to update `install.yml` every new rinstall version)
- Support for both user and system-wide installation
- Works inside the codebase or from the release tarball
- Native support for _Rust_ programs and *cargo*
- Easy uninstallation of packages
- Allow templating of documentation and man pages
- Support for GNU Directory standard, FHS and XDG BaseDirectories, with optional configuration
- Support most common types of files
- Reproducible installation
- Packagers friendly

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

After having reviewed the changes, add `-y` or `--yes` to perform an user installation:

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
- `systemd_unitsdir`

In addition, the system-wide configuration can contain the following keys:

- `prefix`
- `exec_prefix`
- `sbindir`
- `libexecdir`
- `includedir`
- `docdir`
- `mandir`
- `pam_modulesdir`

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
- `@libdir`, supported in `pam_modulesdir` and `systemd_unitsdir`

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
- `@XDG_CONFIG_HOME@`, supported in `sysconfdir` and `systemd_unitsdir`
- `@XDG_STATE_HOME@`, supported in `localstatedir`
- `@XDG_RUNTIME_DIR@`, supported in `runstatedir`
- `@sysconfdir@`, supported in `systemd_unitsdir`

## Writing `install.yml`

To support **rinstall**, place an `install.yml` file into the root of your project. It shall contain
the rinstall version to use and the packages to install. Each package shall contain the
entries of the files to install, divided by their purpose/destination.

Example file for a program named `foo` written in Rust that only install an executable with
the same name:

```yaml
rinstall: 0.1.0
pkgs:
  foo:
    type: rust
    exe:
      - foo
```

### rinstall version

each **rinstall** release will have a respective version of the spec file; each version might
support new entry types but it might remove support for some as well. rinstall will support older
releases, along with all its entry types which were allowed.

### Packages

**rinstall** support the installation of multiple packages from the same repository. Put all the
packages under a unique name inside the key `pkgs` in the `install.yml` file (even if there is only
one package):

```yaml
rinstall: 0.1.0
pkgs:
  foo:
    type: rust
    exe:
      - foo
  bar:
    type: rust
    exe:
      - bar
  bar-c:
    type: custom
    include:
      - bar.h
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

### Valid entries

**rinstall** allows for the following keys:

_Note_: ***each entry will be available for both system and non-system wide installations except
were expliticly noted***.

#### Type

(_since 0.1.0_)

The type part can either be `rust` or `custom`. If no value is specified, then `custom` will be
used.

- `rust` for projects built using `cargo`. The target directory is fetched using `cargo metadata`
  and used as root directory for executables and libraries. I.e. you don't need to use
  `target/release/myexe` when listing executables, but just `myexe`.

- `custom` for all the other projects. All the directories will be relative to the root directory
  of the project.

#### `exe`

(_since 0.1.0_)

For the executables; they will be installed in `bindir` (which defaults to
`/usr/local/bin`)

#### `admin_exe`

(_since 0.1.0_)

_Only available in system-wide installation._

For admin executables; they will be installed in `sbindir` (which defaults to `/usr/local/sbin`).

#### `libs`

(_since 0.1.0_)

For the libraries; they will be installed in `libdir` (which defaults to `/usr/local/lib`).

#### `libexec`

(_since 0.1.0_)

_Only available in system-wide installation._

`libexec` files will be installed in `libexecdir` (which defaults to `/usr/local/libexec`).

#### `include`

(_since 0.1.0_)

_Only available in system-wide installation._

`include` files will be installed in `includedir` (which defaults to `/usr/local/include`).

#### `man`

(_since 0.1.0_)

_Only available in system-wide installation._

For the man pages; they will be installed under the correct folder in `mandir`
(which defaults to `/usr/local/share/man`).

#### `data`

(_since 0.1.0_)

For architecture independent files; they will be installed in `datarootdir` (which
defaults to `/usr/local/share`).

#### `docs`

(_since 0.1.0_)

For documentation and examples; they will be installed in folder
`doc/<pkg-name>` under folder `datarootdir` (which defaults to
`/usr/local/share/doc/<pkg-name>`).

#### `config`

(_since 0.1.0_)

For configuration files; they will be installed in `sysconfdir` (which defaults to
`/usr/local/etc`).

#### `user-config`

(_since 0.1.0_)

_Only available in non-system wide installations._

For configuration files that should only be installed for non-system wide installations; they will
be installed in `sysconfdir` (which defaults to `$XDG_CONFIG_HOME/.config`).

#### `desktop-files`

(_since 0.1.0_)

For `.desktop` files; they will be installed in folder
`applications` under `datarootdir` (which defaults to `/usr/local/share/applications`).

#### `appstream-metadata`

(_since 0.1.0_)

_Only available in system-wide installation._

For [AppStream metadata] files; they will be installed in folder
`metainfo` under `datarootdir` (which defaults to `/usr/local/share/metainfo`).

[AppStream metadata]: https://www.freedesktop.org/software/appstream/docs/chap-Metadata.html

#### `completions`

- *bash* (_since 0.1.0_)
- *fish* (_since 0.1.0_)
- *zsh* (_since 0.1.0_)

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

#### `pam-modules`

(_since 0.1.0_)

_Only available in system-wide installation._

For PAM modules; they will be installed in `@libdir@/security` (`/usr/local/lib/security`
by default). If only `src` is provided, and the name of the file starts with `lib`, e.g.
`libpam_mymodule.so`, it will be automatically converted to `pam_mymodule.so`.

#### `systemd-units`

(_since 0.1.0_)

_Only available in system-wide installation._

For systemd system units; they will be installed in `@systemd_unitsdir@/system` (`/usr/local/lib/systemd/system` by default).
#### `systemd-units`

(_since 0.2.0_)

For systemd user units; they will be installed in `@systemd_unitsdir@/user` (`/usr/local/lib/systemd/user` by default).

#### `icons`

(_since 0.1.0_)

For icons. There two different locations for icons:

- `@datarootdir@/pixmaps` (system-wide only)
- `@datarootdir@/icons`

To install an icon into one or the other, use `pixmaps`:

```yaml
icons:
  - src: myicon.svg
    pixmaps: true
```

The icons in the latter are divided into different folders by:

- `theme`, which defaults to `hicolor`
- `dimensions`, which is the size of the icon in the form of `YxY` (`48x48`) or
  `scalable` for svg icons (**mandatory**)
- `type`, which defaults to `apps`

Example:

```yaml
icons:
  - src: myicon.svg
    dimensions: scalable
```

`theme` and `type` are optional. For more information the entries in `@datarootdir@/icons`, have a
look at the [Directory Layout] of the freedesktop icon theme specification.

[Directory Layout]: https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html#directory_layout

#### `terminfo`

(_since 0.1.0_)

For terminfo sources; they will be installed in `@datarootdir@/terminfo`
(`/usr/local/share/terminfo` by default). The source files won't be compiled by **rinstall**.
Please compile them manually after installation by using `tic`.
The files there are divided into different folders based on the first letter of the file name.
For example the file `alacritty.info` should be installed in
`/usr/local/share/terminfo/a/alacritty.info`. Just use the name of the file in `src` or `dst`
and **rinstall** will handle the directory.

#### `licenses`

(_since 0.1.0_)

For licenses; they will be installed in `@datarootdir@/licenses/<pkg-name>`
(`/usr/local/share/licenses/<pkg-name>` by default).

#### `pkg-config`

(_since 0.1.0_)

_Only available in system-wide installation._

For `pkg-config` files; they will be installed in `@libdir@/pkgconfig`
(`/usr/local/lib/pkgconfig` by default).

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
- `@pam_modulesdir@`
- `@systemd_unitsdir@`

### Release tarballs

**rinstall** supports installing from release tarballs (i.e. the tarballs published on Github
for each release containing a compiled version of the program).

To allow a program to be installed from a release tarball create a `.tarball` empty file during
the generation and include `install.yml`. **rinstall** will then assume that all the files are in
the top directory and proceed to install them as usual; this means that for _Rust_ programs, the
executables will be searched in the top directory instead of `target/release`. Please assure that
all the files listed in `install.yml` are included in the tarball.

### Uninstall

When a package gets been installed, a `.pkg` will be installed inside `localstatedir/rinstall`.
This file will contain the list of files so that when running the `uninstall` subcommand,
rinstall can revert the installation of a package:

```bash
$ rinstall uninstall foo
Would remove "/usr/bin/foo"
```

## License

**rinstall** is licensed under the GPL-3+ license.

