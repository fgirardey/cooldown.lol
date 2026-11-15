# Design

Pencil design files for cooldown.lol. This folder is the **design memory** of the project: Claude Code reads it through the Pencil MCP server when implementing UI.

## Files

| File | Content |
|---|---|
| `cooldown.pen` | Main design file |
| `brand/` | Logo files exported from the `Logo / *` components (see [`docs/Design/Brand.md`](../docs/Design/Brand.md)) |
| `assets/roles/` | League position symbols (`top`, `jungle`, `middle`, `bottom`, `utility`), from CommunityDragon's mirror of the client files, used in the role badges of `Champion portrait`. Also bundled in the app (ADR-0005) |
| `assets/champions/` | Champion square images from Riot's Data Dragon (version 16.22.1), used as image fills by the `Champion portrait` instances. Same source as the app ([ADR-0004](../docs/Decisions/ADR-0004%20Champion%20images%20from%20Data%20Dragon.md)) |

Add more `.pen` files only if the main one becomes hard to navigate (e.g. `overlay.pen` for in-game overlays).

## Conventions

So that Claude (and you, in six months) can find things:

- **One canvas, stacked sections**, top to bottom: `Foundations` (brand and tokens boards), `Primitives` (logo, switch, exercise pictograms), `Controls` (buttons, icon buttons, text links and their states), `Core components` (app shell, champion portrait), `History components` (session headers, game rows, stat pictograms), then the screen rows, and `Explorations` (boards kept for reference, like the rebrand) last. Each section starts with a `Flow label / …` title, and sections never overlap: when a board grows, move the sections below it down.
- **Frame names mirror the feature notes** in `docs/Product/Features/`: `<Feature> / <Screen> / <State>`, e.g. `Champ Select / Main / Loading`. Every state the UI can be in gets a frame (empty, loading, error, nominal).
- **Real sizes**: frames use the actual window or overlay size in logical pixels. Note the target resolution in the frame name when it matters (`… @1080p`).
- **Variants inherit from a base**: when a component has several states, build one `<Component> / Base` component holding the union of their nodes (optional ones disabled), and make each state an instance of it that only overrides what differs: content, colours, which optional nodes are shown. Changing the base then changes every state and every screen. Content that only one state has (the details of an expanded row) goes in a slot of the base that the state replaces. Example: `Game row / Games / Base` and `Game row / Workouts / Base` in `History components`.
- **Tokens first**: colors, spacing, radius and type sizes come from the `Foundations` board and are mirrored in [`docs/Design/Tokens.md`](../docs/Design/Tokens.md). Do not use one-off values.
- **Flow layout**: screens are laid out in rows following the user flow (`Flow label / …` titles, arrows labelled with the action that leads to the next frame), below the primitives and core components. See `docs/Product/Screens.md#Frames`.
- **Annotations**: write behaviour that a static frame cannot show (animations, hover, what updates live) as text notes next to the frame, or in the feature note.
- **No Riot assets** (logos, splash art, icons) in committed designs unless their use is allowed — see [`docs/Compliance/Riot policies.md`](../docs/Compliance/Riot%20policies.md). Use neutral placeholders. Exceptions: champion square images from Data Dragon in `assets/champions/` (ADR-0004) and the position symbols in `assets/roles/`, the same kind of assets the app shows. Never Riot logos.

## Git

`.pen` files are encrypted: read and edit them only through Pencil (the app or its MCP server), never as text. The Pencil app must be running with the file open for Claude to see the designs. Commit them like code, with a message describing the design change. Do not merge conflicting `.pen` files by hand: keep one version and reapply the other change in Pencil (`.gitattributes` marks them as non-mergeable).

See [`docs/Workflow/Design with Pencil.md`](../docs/Workflow/Design%20with%20Pencil.md).
