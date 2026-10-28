# Third-Party Notices

P12 Librarian includes libraries, colour themes and fonts made by others,
under their own licences. P12 Librarian itself is under the MIT License (see LICENSE);
nothing in its licence limits the rights these licences give you.

## Libraries

The licences of the Rust and JavaScript libraries built into the app and the
`p12` tool are listed in full in `licenses/rust.md` and
`licenses/javascript.md` (in the app: Help → Licences; the Debian package
installs them in `/usr/share/doc/p12-librarian/licenses/`). The five libraries
under the Mozilla Public License 2.0 (cssparser, cssparser-macros, dtoa-short,
option-ext and selectors) are used unmodified; their source code is available
from crates.io.

## Colour themes

The colours of these themes are taken from the theme files in
[kitty-themes](https://github.com/kovidgoyal/kitty-themes) and their upstream projects,
and adapted to the app (see `app/src/app.css`). No theme files are included. Every theme
listed is licensed under the MIT License, reproduced below.

| Theme | Author | Upstream |
|---|---|---|
| Dracula | Dracula Theme; kitty port by Keegan Carruthers-Smith | <https://github.com/dracula/kitty> |
| Catppuccin Mocha, Catppuccin Latte | Catppuccin; kitty port by Pocco81 | <https://github.com/catppuccin/kitty> |
| Tokyo Night | Folke Lemaitre | <https://github.com/folke/tokyonight.nvim> |
| Gruvbox Dark | Pavel Pertsev; kitty port by gruvbox-community | <https://github.com/gruvbox-community/gruvbox-contrib> |
| One Dark Pro | VictorPL, after Binaryify's One Dark Pro | <https://github.com/VictorPLopes/OneDark-Pro-Kitty-Terminal> |
| Rosé Pine | mvllow | <https://github.com/rose-pine/kitty> |
| Kanagawa | Tommaso Laurenzi | <https://github.com/rebelot/kanagawa.nvim> |
| Everforest Dark | Sainnhe Park | <https://github.com/ewal/kitty-everforest> |
| Solarized Dark | Ethan Schoonover | <https://ethanschoonover.com/solarized/> |

### MIT License

Copyright (c) the theme authors named above

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Fonts

JetBrains Mono and Hack are the unmodified [Nerd Fonts](https://www.nerdfonts.com) v3.5.1
builds; Fira Code comes from the `@fontsource/fira-code` package. The full licence texts
are next to the font files, and the Debian package installs them in
`/usr/share/doc/p12-librarian/fonts/`.

| Font | Copyright | Licence | Files |
|---|---|---|---|
| JetBrains Mono Nerd Font | Copyright 2020 The JetBrains Mono Project Authors | SIL Open Font License 1.1 | `app/src/assets/fonts/jetbrains-mono/` (`OFL.txt`) |
| Hack Nerd Font | Copyright 2018 Source Foundry Authors; Bitstream Vera Sans Mono Copyright 2003 Bitstream Inc.; DejaVu changes in the public domain | MIT License and Bitstream Vera License | `app/src/assets/fonts/hack/` (`LICENSE.md`) |
| Fira Code | Copyright 2014-2020 The Fira Code Project Authors | SIL Open Font License 1.1 | via the `@fontsource/fira-code` package |

The Nerd Fonts icon glyphs come from their respective icon projects, as listed in the
Nerd Fonts licence: <https://github.com/ryanoasis/nerd-fonts/blob/master/LICENSE>.
