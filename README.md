# duckyctl

A utility to control duckyPad and duckyPad Pro over HID

---

The [duckyPad](https://dekunukem.github.io/duckyPad/) and its successor, the [Pro](https://dekunukem.github.io/duckyPad-Pro/), are keyboard macropads that are configured using a scripting language known as DuckyScript instead of VIA/QMK. DuckyScript is a fully Turing-complete language, and so these pads are much more powerful than any macropad that uses QMK/VIA. Click on the links above for more information, or [this page](https://dekunukem.github.io/duckyPad-Pro/doc/duckyscript_info.html) to learn about DuckyScript.

The duckyPad (Pro) can be controlled from the host machine via the USB HID protocol to perform certain actions, such as changing profile or sleeping/waking. `duckyctl` is a small utility that provides a CLI for controlling the duckyPad this way.

## Usage

See the [HID documentation](https://github.com/duckyPad/duckyPad-Profile-Autoswitcher/blob/master/HID_details.md) for information about how HID communication works, and what commands are available.

Each HID command is exposed as a subcommand. Arguments to these commands follow each subcommand. For example, to send the HID "Go to profile 2" command, you would execute:

```bash
$ duckyctl goto-profile-num 2
```

For more information on the arguments available to each subcommand, run `duckyctl <subcommand> --help`.
