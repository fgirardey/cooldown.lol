# Design tokens

Single reference for the visual foundations. **Mirrored 1:1** in `crates/cooldown-app/ui/theme.slint` (global `Theme`) and in the `Foundations` board of `design/cooldown.pen`.

When a value changes: update Pencil, this note and `theme.slint` in the same commit.

Colours come from the brand direction chosen in Pencil (see [[Brand]]): a K/DA reading where green is what you gained and red what you gave away.

## Colors
| Token | Value | Usage |
|---|---|---|
| `background` | `#0d1117` | Window background |
| `surface` | `#161b22` | Cards, panels |
| `border` | `#30363d` | Dividers, outlines |
| `text-primary` | `#e6edf3` | Main text |
| `text-secondary` | `#8b949e` | Secondary text, labels |
| `accent` | `#3fb950` | Brand accent, highlights |
| `success` | `#3fb950` | Kills, gains, connected |
| `danger` | `#f85149` | Deaths, losses, disconnected, errors, destructive actions (Uninstall) |
| `warning` | `#d29922` | Partial workouts, sessions with some workouts missed, a game's cap applied |
| `info` | `#58a6ff` | Wins (result, stripe and tint of a History row, "W" chip) |
| `scrim` | `#010409c7` | Dims the window behind a dialog (workout prompt, uninstall confirmation) |
| `surface-raised` | `#262c36` | Panels floating above the screen (tooltips): a lighter tone than `surface`, as Material shows elevation in dark mode |
| `border-raised` | `#444c56` | Outline of a raised panel |
| `shadow` | `#01040999` | Shadow under a raised panel |

`accent` and `success` share a value on purpose (green is the brand colour), but keep the roles separate in code.

## Spacing
| Token | Value |
|---|---|
| `space-xs` | 4px |
| `space-s` | 8px |
| `space-m` | 16px |
| `space-l` | 24px |
| `space-xl` | 32px |

## Radius
| Token | Value |
|---|---|
| `radius-chip` | 4px (session header chips, tooltip) |
| `radius-s` | 6px (view switch segments, row chevron) |
| `radius-m` | 8px |
| `radius-dialog` | 16px |
| `radius-panel` | 10px (session panel, champion portrait) |

## Session panel
Tints of a session panel ([[History#Session summary]]): the session's colour (`text-secondary` for a session of games from before cooldown.lol) at these opacities. They are Pencil variables, but the `Session header / *` components still use literal values.

| Token | Value | Usage |
|---|---|---|
| `tint-header` | 0.12 | Summary header background; the difference chip of a cost curve's tooltip |
| `tint-line` | 0.4 | Line under the summary header |
| `tint-games` | 0.04 | Games area background |
| `muted-opacity` | 0.75 | "No workout" card |

## Game row
Tints of a History row in the Games view ([[History#Game rows]]): the result's colour fades from the stripe on the left, at these opacities at 0 % and 28 % of the row's width, and is gone at 60 %.

| Token | Value | Usage |
|---|---|---|
| `tint-row` | 0.22 | Win or loss tint, at the stripe |
| `tint-row-mid` | 0.08 | Win or loss tint, at 28 % |
| `tint-row-muted` | 0.15 | `text-secondary` tint of a game without workout, at the stripe |
| `tint-row-muted-mid` | 0.05 | Same, at 28 % |
| `tint-curve` | 0.25 | `danger` → `warning` → `success` gradient between a cost curve and its 0 line, in the row's details |
| `tint-gauge-zone` | 0.50 | `danger` → `warning` → `success` gradient of a benchmark gauge in a Games row |

Lines of a cost curve in a History row's details ([[History#Game rows]]): `text-secondary` at these opacities, except the halo, in the zone colour of the game's dot.

| Token | Value | Usage |
|---|---|---|
| `curve-zero-line` | 0.6 | The 0 line, at the top |
| `curve-grid` | 0.25 | Dotted line at the mid amount |
| `curve-target-line` | 0.5 | Vertical line at the target |
| `curve-projection` | 0.6 | Dashed lines from the game's dot to the axes |
| `curve-halo` | 0.2 | Halo around the game's dot |
| `watermark-opacity` | 0.3 | The stat's pictogram and name behind the curve |
| `tint-tooltip-game` | 0.12 | The zone colour mixed into the panel of the game's own tooltip ("THIS GAME"), pointer included |
| `tint-tooltip-game-outline` | 0.7 | Its outline, in the zone colour |
| `tint-tooltip-game-divider` | 0.35 | Its divider, in the zone colour |

## Interaction
Hover, pressed, focus, disabled and selected states. Buttons, icon buttons and text links are Pencil components in `Controls library`, with one reusable instance per state (`Button / Primary / Hover`, …) and a **State rules** table (trigger, visual change, tokens, motion, cursor); switches and workout cards are in the `Interaction states` section of the `Foundations` board. Every interactive control adds a **state layer** in its content colour (Material), filled and outlined buttons also **shrink** while pressed (iOS), and every change eases out.

| Token | Value | Usage |
|---|---|---|
| `state-opacity-hover` | 0.08 | State layer opacity while hovered |
| `state-opacity-pressed` | 0.12 | State layer opacity while pressed |
| `press-scale` | 0.97 | Scale of filled and outlined buttons while pressed |
| `disabled-opacity` | 0.4 | Opacity of a disabled control (e.g. **Check now** while a check runs) |
| `motion-fast` | 100ms | Hover and press transitions (ease-out) |
| `motion-medium` | 200ms | Switch toggle, card expand and collapse, screen transitions (ease-out) |
| `motion-push-offset` | 30px | How far a pushed screen slides in from, to the right ([[ADR-0018 Navigation stack]]) |
| `motion-type-erase` | 10ms | Top bar headline: the cursor erases one character ([[ADR-0018 Navigation stack]]) |
| `motion-type-char` | 18ms | Top bar headline: the cursor types one character |
| `motion-type-phase-max` | 150ms | Top bar headline: longest erase or type phase; longer titles go faster |
| `motion-cursor-hold` | 120ms | Top bar headline: how long the cursor stays after the last character |
| `focus-ring` | `#58a6ff` | Keyboard focus ring (Tab only, never on click) |
| `focus-ring-width` | 2px | Focus ring thickness, drawn outside the control and following its radius; replaces the border of outlined buttons |

The state layer colour is the control's content colour: `background` on `accent` buttons and on the "on" switch track, `text-primary` everywhere else. Text links have no layer: they turn `text-primary` on hover and drop to 60% opacity while pressed. In Pencil the durations are plain numbers in milliseconds.

In Slint, `components/state-layer.slint` (`StateLayer`) draws the layer: every interactive control lays one over its background, fed by its `TouchArea`'s `has-hover` and `pressed`.

## Tooltip
A control whose state needs a reason (a disabled button, a running task) explains it in a **tooltip**, and a control with no label (an icon button, a badge) is named by one. In Pencil it is the `Tooltip` component of `History components`. In code ([[ADR-0017 Tooltips drawn by the window]]):
- wrap the control in `TooltipAnchor` (`components/tooltip-anchor.slint`) with its `tooltip` text;
- the window's `TooltipLayer` (`shell/tooltip-layer.slint`) draws it as `TooltipText` (`components/tooltip-text.slint`), a text in a `TooltipPanel`.

Disabled controls show it too: the anchor goes around the faded part.

Look: `surface-raised` panel with a 1px `border-raised` outline and a `shadow` (`shadow-raised-blur`, `shadow-raised-offset`), `radius-chip`, padding 6px × 10px, text in `font-size-caption` and `text-primary`, wrapped at `tooltip-max-width`. A **pointer** on the edge facing the control aims at its centre (or at the spot a chart's tooltip explains), kept 12px off the corners. Panel and pointer are one outline (`TooltipSurface`: one fill, one continuous stroke), with the shadow cast by a backdrop behind it. In dark mode Material shows elevation with a lighter container tone and a shadow rather than a stronger outline (its rich tooltip is `surface-container` at elevation level 2, its plain tooltip `inverse-surface`); the raised tone follows that, while staying dark.

Timing and placement follow Material Design's tooltip (MDC Web values):

| Token | Value | Usage |
|---|---|---|
| `tooltip-show-delay` | 300ms | The pointer rests this long on the control before the tooltip opens (quicker than MDC Web's 500ms, still long enough not to flash tooltips at a pointer crossing controls) |
| `tooltip-hide-delay` | 100ms | The tooltip stays this long after the pointer leaves, just enough to reach a neighbouring control and switch the text at once (MDC Web's 600ms lingered too long) |
| `tooltip-enter` | 150ms | Fades in and grows from `tooltip-enter-scale`, from the control's side (deceleration curve `cubic-bezier(0, 0, 0.2, 1)`) |
| `tooltip-exit` | 75ms | Fades out (acceleration curve `cubic-bezier(0.4, 0, 1, 1)`) |
| `tooltip-enter-scale` | 0.8 | Scale the tooltip grows from |
| `tooltip-gap` | 4px | Between the control and the tip of the tooltip's pointer |
| `tooltip-window-margin` | 8px | The tooltip keeps this far from the window edges |
| `tooltip-max-width` | 220px | Width of the text before it wraps |
| `tooltip-pointer` | 7px | How far the pointer sticks out toward the control (a triangle twice as wide); the panel sits this much further from the control |
| `shadow-raised-blur` | 16px | Blur of a raised panel's shadow |
| `shadow-raised-offset` | 6px | How far down a raised panel's shadow falls |

Placement:
- **Vertically:** under the control, pointer on top; above it when there is no room below, pointer at the bottom, keeping `tooltip-window-margin` from the edges. When neither side has room (a tall tooltip in a short window), on the side with more room, kept inside the window even if it covers the control.
- **Horizontally:** centered on the control. When centering would get closer to a window edge than the margin, aligned with the control's start, then with its end. As a last resort, kept off the edges. Wherever the panel goes, its pointer stays aimed at the control.
- **Narrow window:** when the window is narrower than the tooltip plus both margins, the tooltip narrows to fit between them and its text wraps more.

The tooltip stays inside the window. It never takes clicks or hover, and it closes when the pointer leaves the window or the screen changes.

## Typography
| Token | Size | Usage |
|---|---|---|
| `font-size-micro` | 10px | A game's max ("max 15"), the scale of a benchmark gauge |
| `font-size-small` | 11px | Axis labels of a cost curve in a History row's details |
| `font-size-caption` | 12px | Small labels (top bar status) |
| `font-size-label` | 13px | Workout status, view switch, exercise names |
| `font-size-body` | 14px | Body text |
| `font-size-wordmark` | 18px | "COOLDOWN.LOL" wordmark in the top bar |
| `font-size-title` | 20px | Titles, stat values of History rows |
| `font-size-result` | 22px | Results of the Games view, the game's value on a cost curve, "/due" and the exercise in a row's details |
| `font-size-watermark` | 30px | The stat's name behind a cost curve |
| `font-size-headline` | 32px | Welcome headline |
| `font-size-display` | 40px | Big numbers (clock, stats) |
| `font-size-hero` | 96px | The one number to read from a distance (workout step count, plank countdown) |

Font families (Pencil variables `font-display` and `font-body`):

| Role | Family | Usage | License |
|---|---|---|---|
| Display | Barlow Condensed (ExtraBold/Bold Italic) | Wordmark, titles, big numbers | SIL Open Font License 1.1 |
| Body | Inter | Body text, labels | SIL Open Font License 1.1 |

The fonts are bundled in `crates/cooldown-app/ui/fonts/` with their `OFL.txt` (Barlow Condensed ExtraBold Italic, Bold Italic, ExtraBold and SemiBold, the weights the design uses; Inter as a variable font) and imported from `theme.slint`, which mirrors the two families as `font-display` and `font-body`. The windows set `default-font-family` to `font-body`, and every `Text` sets its family explicitly so previews render like the app: `font-display` for the wordmark, titles, results and big numbers, `font-body` for the rest.

## Naming rules
- kebab-case, same name everywhere (Pencil variable, this table, `Theme.<name>` in Slint).
- Name by role (`text-secondary`), not by look (`grey-400`).
