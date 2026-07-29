# Rules data — licensing and attribution

This `rules` folder ships with the Ars Magica Character Generator and holds the
game data the application loads at startup. It contains **two differently
licensed bodies of work**, and this notice is the attribution that travels with
them.

In short: the **format** is the project's own work and is MIT-licensed; the
**Ars Magica rules text** inside it belongs to Atlas Games and is licensed under
CC BY-SA 4.0. Reusing the format needs only MIT. Redistributing or adapting the
rules text keeps it under CC BY-SA 4.0 with attribution preserved.

## `core/` — mechanics data (MIT)

Wholly MIT-licensed. These files carry no rulebook text: only slug identifiers,
numbers, enums, prerequisite and grant structures, and citations recording which
source file and line range each value came from. Where a slug echoes a rulebook
term it is used as an identifier, not as expressive content.

## `i18n/` — localized rules text (MIT form, CC BY-SA 4.0 text)

Dual licensed:

- The **form** is MIT — the JSON schema, the object and key names, the stable
  slug-identifier scheme the text is keyed by, and the file layout.
- The **string values** are CC BY-SA 4.0 — every name, description, summary and
  specialty under `i18n/<lang>/` is extracted from the Ars Magica rulebooks and
  is in places verbatim rulebook prose. Translations of that text are
  derivatives and are CC BY-SA 4.0 as well.

## The MIT License

Copyright (c) 2026 Ars Magica 5th Edition Character Generator contributors

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

## Attribution for the rules text

Based on the material for Ars Magica, (c) 1993-2024, licensed by Trident, Inc.
d/b/a Atlas Games, under the Creative Commons Attribution-ShareAlike 4.0
International License ("CC BY-SA 4.0"):

    https://creativecommons.org/licenses/by-sa/4.0/

This is commonly referred to as the **Ars Magica Open License**.

The Open License Markdown editions this data was extracted from originate from:

- English: applejuice1965 & OriginalMadman (and contributors) —
  https://github.com/OriginalMadman/Ars-Magica-Open-License —
  via the fork at https://github.com/garin1000/Ars-Magica-Open-License
- German: garin1000 (and contributors) —
  https://github.com/garin1000/Ars-Magica-Open-License-German

## Share-alike

Under CC BY-SA 4.0, any adapted or derived version of the rules text must be
distributed under the same license with attribution preserved. This applies to
the rules text in `i18n/<lang>/` and to translations of it.

Shipping that text alongside the MIT-licensed application in a single installer
or archive is a **collection, not an adaptation**: it does not place the
application under CC BY-SA 4.0.

## Trademarks

Ars Magica, Mythic Europe, and the various sourcebook titles are trademarks of
Trident, Inc. d/b/a Atlas Games. Order of Hermes, Tremere, Doissetep, and
Grimgroth are trademarks of Paradox Interactive AB and are used with permission.
These trademarks are **not** licensed under CC BY-SA 4.0; the license covers the
textual content only, not the trademarks.

## Not affiliated

This is an unofficial fan tool. It is not affiliated with, sponsored by, or
endorsed by Atlas Games.

Project: https://github.com/garin1000/Ars-Magica-Character-Generator
