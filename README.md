# display-switch-tray

A tray icon for toggling [display-switch](https://github.com/haimgel/display-switch). This is done by just running systemctl enable/disable, and you will probably need to add a sudoers file for this to work (see [extra/60_display_switch](extra/60_display_switch))

A package is available in the [Arch User Repository](https://aur.archlinux.org/packages/display-switch-tray).

## TODO:

I plan on implementing a periodic timer that gets the current state, so that it will be updated in case it's disabled through the console or fails.
