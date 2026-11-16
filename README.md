# Hangar

Every Steam account, one click away. A small native app for Windows 10 and 11, by PG.

**[Download the latest release](https://github.com/lilpatter/hangar/releases/latest)**: `Hangar_<version>_x64-setup.exe` installs it, `hangar.exe` runs without installing.

More at [lilpatter.github.io/hangar](https://lilpatter.github.io/hangar/).

<p align="center"><img src="screenshots/home.jpg" width="100%" alt="Home with a featured game, playtime and Continue Playing for the signed-in account." /></p>

## What it does

- **Silent account switching.** Changes the signed-in Steam account without the account picker or a password prompt, and sends Steam straight to the tray.
- **Play as…** A game owned by another account on this PC gets a *Play as <name>* button: one click switches accounts and launches it.
- **Your whole library.** Every game on every account, installed or not, in a grid of Steam's own cover art or a list with playtime, last played and size. Filter, sort, and look at one account or all of them.
- **Favorites**, shared by every account.
- **Friends, chat and profiles.** See who's online and what they're playing, read and send messages with your full history, and open profiles with their games, games in common and comments. Sign in by approving a QR code in the Steam Mobile App. Hangar's chat runs as its own session, so it never changes your online status.
- **Steam stays out of the way.** Hangar starts Steam in the tray, with no windows, and games launched from Hangar do the same.
- **Less of Steam in your RAM.** While Steam sits idle, Hangar asks Windows to trim its working set, the part of its memory held in RAM: about 330 MB down to 30–70 MB, measured on Windows 11 with Steam idle in the tray. Steam's total allocated memory stays the same; trimmed pages come back if Steam needs them. Never while a game runs.
- **Optional Steam Web API key** adds games an account owns but has never opened on this PC.

Hangar never reads or stores passwords. An account switches silently once it has signed in on this PC with *Remember me* checked. The friends sign-in is kept encrypted with Windows' own data protection, and you can revoke it under Steam's *Authorized Devices*.

## Screenshots

<p align="center">
  <img src="screenshots/games.jpg" width="49%" alt="Installed games in a grid of Steam cover art, with playtime for each." />
  <img src="screenshots/accounts.jpg" width="49%" alt="Three remembered Steam accounts, each with a Switch button, and Add Account." />
</p>

## Requirements

Windows 10 or 11 with Steam and the WebView2 runtime, which ships with Windows 11.

Hangar is a Windows app. There's no Mac version yet: the Mac-style sidebar and window buttons are just part of its design.

## Updates

Hangar updates itself from this page's releases and checks each download against GitHub's SHA-256 digest. Turn it off under Settings › About.

## Credits

Made by [PG](https://lilpatter.github.io).

Not affiliated with Valve. Steam is a trademark of Valve Corporation.

---

© 2026 PG. All rights reserved. Hangar is free to download and use. Its source code is not public.
