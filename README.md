DUX
===
An X11 backlight manager.

Installation
------------
To install it you will need a nightly Rust toolchain, then you can install it
with Cargo:

```shell
cargo install dux
```

Usage
-----
`dux` can be used like a replacement for `xbacklight`, the command syntax changes
slightly but the functionality is the same (`get`, `set`, `inc`, `dec`, all
with the usual fade settings).

To start the adaptive brightness daemon just run:

```
dux adaptive &
```

To stop it gracefully (making sure the settings are saved) just run:

```
dux stop
```

Adaptive brightness
===================
Adaptive brightness manages the backlight automatically for you based on the
selected mode and profile.

To select a mode you can either pass a `--mode <mode>` when starting the
adaptive brightness, or call `dux mode <mode>` after it's been started.

There's support for multiple profiles, to select a profile just pass `--profile
<name>` when starting the adaptive brightness, or call `dux profile <name>`;
profiles are useful for example to have different settings during the night and
during the day, or when you're inside or outside.

To configure the brightness levels for the various modes all you have to do is
change the backlight from `dux` itself like you would with `xbacklight` and the
change will be saved. If you don't want to do that you can call `dux sync`
after changing the backlight with something else.

Desktop
-------
The `desktop` mode uses the current active desktop (also known as workspace in
some window managers) to reload the previously set brightness.

Window
------
The `window` mode uses the active window to to reload the previously set
brightness.

Tt uses both the window's instance and class name to determine the brightness,
this allows for a common brightness setting for the class and a specific one
for the named window.

Luminance
---------
The `luminance` mode uses the screen content's contrast to reload the
brightness value.

When the luminance is between two different settings it will interpolate the
brightness value between the two based on the distance between them.

For example if you have `10` luminance set at `80` brightness and `50`
luminance set at `20` brightness and the current luminance value is `20` the
brightness will be closer to `80` than `20`.

Performance wise it uses some X extensions to avoid doing heavy work, it uses
the MIT-SHM extension to avoid connection pressure when fetching the screen
contents and the DAMAGE extension to only fetch and recalculate the areas that
have actually changed.
