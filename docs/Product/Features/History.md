---
status: in progress (games from before cooldown.lol; the workout flow is not built yet)
moment: always
design: "sketched: History / Main / *"
---

# History

Every game cooldown.lol has seen, **and the games the player played before installing it**, grouped by session, with the workout that followed each game. It is the app's **main screen**: the workout itself opens as a modal over it (see [[Post-game workout#Workout modal]]). Inspired by the session view of DPM.LOL's match history (games grouped under a session header with a summary), but the workout comes first and the game second.

## User story
As a player, I want to look back at my sessions and see, game after game, which workout I did, partly did or skipped, so that I keep myself honest across days.

## Why
The workout only works as a habit if the player sees their streak, and their skips (see [[Vision#Success]]).

## Behaviour

### Sessions
- A **session** is a run of games where each game starts less than **1 hour** after the previous one ended. A longer break starts a new session. App restarts do not matter.
- Session label: day and rank of the session in that day, e.g. "Today · #1", "Yesterday · #2", "Sun 1 Nov · #1".
- Sessions are listed newest first. The current session (if any) is on top.

### Views
The screen's title row holds "HISTORY" and a **view switch**. There is no legend: tooltips explain what needs explaining. The two views list the same sessions and games; only the middle of each row changes:
- **Games** (default, swords icon): the stats of each game.
- **Workouts** (dumbbell icon): the exercises each game turned into.

The view picked stays while the app runs; it is not saved.

### Refresh
At the right end of the title row: when History was last read from the League client, and a **refresh button** (32 px square, `surface` with a `border` outline, the Lucide refresh icon in `text-secondary`) that reads it again now (see [[#Games from before cooldown.lol]]).
- Label, `text-secondary`: "Updated just now", "Updated 2 min ago", "Updated 3 h ago", "Updated 2 d ago", counted from the end of the last read in this run of the app and kept current while the window is open. Nothing before the first read.
- While a read runs (from the button, at connection or after a game): "Updating…", button disabled.
- Without the League client: button disabled, the label stays.
- History also refreshes on its own, so the button is for when the player does not want to wait (or doubts it worked). There is no timer: nothing counts down to the next read.
- The button's **tooltip** always says why it is in its state, disabled or not ([[Tokens#Tooltip]]):

| State | Tooltip |
|---|---|
| Idle | "Read your latest games from the League client now. History also updates on its own after each game." |
| Reading | "Reading your latest games from the League client…" |
| Waiting for the player to log in (the read tries again every 10 s) | "Waiting for you to log in to the League client. Trying again every 10 s." |
| The client failed to answer (same) | "The League client isn't answering. Trying again every 10 s." |
| No League client | "Open the League client to refresh History." |

### Session header
Each session starts with a **header** line, not a panel: the label (display font, e.g. "TODAY · #1") on the left, then chips on the right:
- a summary chip: "x of y workouts done" in the session's colour on a tint of it, partial workouts counting as not done (see the table); for a session of games from before cooldown.lol, "5 games · before cooldown.lol" in `text-secondary` on `surface`;
- a wins and losses chip: "1 W" (`info`) and "2 L" (`danger`) on `surface`; remakes count in neither.

Summary chip colour:
| Session | Colour |
|---|---|
| Current session (still in progress) | `accent` |
| Every workout done | `success` |
| At least half done | `warning` |
| Less than half done | `danger` |

### Game rows
Under its header, one **row** per game, newest first: 64 px high, full width, `surface`, rounded. On the left a 3 px **stripe**; in the Games view a tint of the same colour fades from it to nothing at 60 % of the width (`info` for a win, `danger` for a loss, `text-secondary` for a remake or a game without workout; tints in [[Tokens]]).

- **Game** (both views): the played champion portrait with its role symbol and the linked champion ([[Champion portraits]]), the champion's name (Workouts view only: the portrait is enough in the Games view), then the queue ("Ranked Solo/Duo", Games view) or the role ("Bottom · with Lulu", Workouts view), then the result (WIN `info`, LOSS `danger`, REMAKE `text-secondary`) with the game's length ("34:12"), preceded by the K/D/A in the Workouts view. In the Games view the result leads: the result in the display font at 22 px, the length beside it (13 px semibold, `text-primary`) and then when the game was played (its end), relative and abbreviated in `text-secondary` ("31:40 · 2h ago", "· 5m ago", "· 3d ago"); the queue sits on a second line, 13 px medium in `text-primary`; hovering it shows the exact date and time in a tooltip ("Sunday 1 November 2026, 20:35", `Game row / Played / Tooltip`).
- **Games view**, five equal columns:
  - K/D/A in the display font, deaths in `danger`, with the KDA ratio ("1.1 KDA" = kills plus assists over deaths, deaths counted as 1 when 0);
  - CS per minute, damage share, kill participation, vision per minute: pictogram, value, a gauge, and under them the stat's name followed by the game's **total** in `text-primary`, 12 px semibold: total CS ("269"), total damage to champions ("21.3k"), kills taken part in over the team's kills ("7/11"), vision score ("22"). The gauge is full at 10 CS per minute, 3 vision per minute and 100 % for the shares. The stats the [[Workout formula]] judges for the role (main and secondary) get a **benchmark gauge** instead, running from the role's floor (left end) to its target (right end): a track at `tint-gauge-zone` with a blended gradient (`danger` at the floor, `warning` halfway to the average, `success` from the average on) and a `text-primary` **tick** at the target. The gauge shows the scale, not the game: the value takes its zone's colour instead. Hovering the row shows the scale on the gauge: the floor above its left end and the target above its right end, small in `text-secondary`, e.g. "5.5" and "9.0" (`Game row / Games / Hover`). The others keep the plain gauge and stay `text-primary`. A game whose stats could not be read shows "—".
- **Workouts view**, three equal columns, one per exercise: the stat that pays for it (pictogram and name), its value (in its zone's colour: `danger` at the floor or worse, `warning` up to the average, `success` from the average on), "/ target" and the gap to the target ("−72 CS", "−9 pts", see [[Workout formula#Gap to target]]); an arrow; then the amount: pictogram, number done (display font) and "/due", a progress bar, and "max 15" in `warning` when the stat was at its floor or worse. The plank's stat is the deaths, "/ 3" (the deaths target).
- **Workout status**: status icon and label — *Done* (`success`), *Partial* (`warning`), *Skipped* (`text-secondary`), *Pending* (no status label: only a small **START** button with a dumbbell icon, its label "START" in the display font (16 px, extra bold italic) in `background`, which reopens the workout modal where it was left, see [[Post-game workout#Workout modal]]) — or "NO WORKOUT" and the reason.
- A **chevron** at the end opens the row's **details** below it, and closes them; its tooltip says which: "Show details", "Hide details". Details: one line per exercise. Each exercise line holds:
  - the stat, as a watermark on the left of the cost curve's area, vertically centred in it and behind the grid: a large pictogram (32 px) and the name in the display font (30 px, extra bold italic), both at 30 % opacity. The exercise is only named in the result, not repeated here;
  - the **cost curve**, over most of the line: the amount the [[Workout formula]] gives for every value of the stat, from the worst value on the left to the best on the right. The y axis is upside down so that improving reads as climbing: 0 at the top (a solid line, its label in `text-primary`), the max at the bottom, a dotted line at the mid amount. The curve, with rounded bends, sits at the max up to the floor, climbs to the mid amount at the average and to 0 at the target, then stays on the 0 line; deaths run from many to few so that every curve reads the same way. The x axis shows the floor, the average and "target X" (with a thin vertical line); the y axis, on the right, shows 0, the max and the game's amount (a label too close to 0 or the max gives way). The area between the 0 line and the curve (the amount due for every value) is tinted at `tint-curve` with a horizontal gradient that blends at the benchmarks: `danger` up to the floor, fading into `warning` between the floor and the average, then into `success` just past the average. The game is a dot on the curve in its zone's colour with a soft halo of it, with neutral dashed lines down to the x axis and across to its amount; the game's value sits by the dot in the display font and the zone's colour, centred above it, or below it when there is no room above;
  - the result, in a fixed-width column so the lines align: the exercise's 56 px pictogram, then in the display font the number done (40 px, `text-primary`) with "/due" beside it (22 px, `text-secondary`), the exercise under them in uppercase ("BURPEES", 22 px, `text-primary`), and to their right the status as a 24 px symbol only, aligned across the lines: complete (`circle-check`, `success`), missing some (`circle-dot`, `warning`), skipped (`circle-x`, `text-secondary`). A skipped exercise has its number done and its name in `text-secondary`.
- **Hovering a cost curve**, close to the curve, shows what another value would have cost: a marker follows the pointer along the curve, with dashed lines to both axes and its value and amount on them, and a structured tooltip ([[Tokens#Tooltip]] for its timing and placement, with a pointer on its edge aimed at the marker, `tooltip-pointer`) shows, for the value under the pointer:
  - the value in its zone's colour (display font) and the stat ("27 %  damage share");
  - its zone, a dot in the zone's colour and a label: "At the floor or worse", "Below average", "Average or better", or "Target reached" once the exercise is waived;
  - under a divider, the exercise's pictogram and the amount ("5 burpees", "15s plank"), and on the right a chip with the difference with the game: "−6 vs this game" in `success`, "+4 vs this game" in `danger`, "Same as this game" outlined in `text-secondary`.

  The marker follows the pointer exactly, except within 10 px of a marker of the curve (the game's dot, the floor, the average or the target), where it snaps to that marker's value: on the game's dot the tooltip shows the game's own value and amount ("Same as this game"). The hover only takes over when the pointer is within 32 px of the curve: there the system cursor is hidden and the hover marker stands for it; elsewhere on the chart the usual cursor stays and nothing is drawn. Once snapped, the marker's own axis labels turn bold `text-primary` (the benchmark's value on the x axis, the max or 0 on the y axis), and on the game's dot a 20 px ring surrounds the dot instead of the marker (`Game row / Details / Exercise / Hover snapped`). The game's value by its dot stays inside the plot when the game is off the scale. The value is rounded like the stat is shown. The amount is the formula's at that value, rounded up like the game's. Nothing but the game is drawn while the pointer is elsewhere.
- While the end-of-game stats are read, a "Game over · Computing your workout…" row sits on top of the current session.
- A game of a mode out of scope (ARAM, arena, custom games, practice tool…) is **not shown at all**: no row, no place in a session, and it does not bridge two sessions (see [[Post-game workout#Scope]]).
- A game without a workout (remake, role not detected, stats unreadable) has a muted row: `text-secondary` stripe and tint, the status "NO WORKOUT" and the reason. It has no details and is not counted in the session.

### Games from before cooldown.lol
When cooldown.lol connects to the League client, it reads the **active summoner** and the client's **match history** (the player's own recent games) and lists the games it has not stored yet. This is the only way to show history from before the installation.
- Such a game is **imported**. cooldown.lol was not there to offer the workout, so it is shown as **skipped**: the workout is computed with the [[Workout formula]] from the game's stats in the match history, and the game gets the usual *Skipped* row (Workouts view: "0/5", "0/15", "0s/30s"…; details: the ⊗ symbol). The played champion portrait has no linked champion (the match history does not give it), and the Workouts view shows the role alone.
- An imported game whose role Riot did not detect cannot be computed: it shows the muted "No workout" row with the reason "Couldn't detect your role", like a tracked game.
- Imported games group into sessions like any other game (same 1-hour rule), but they **never count in the session summary**: not in the "x of y workouts done" chip or its colour. A session made only of imported games has a **neutral header**: label ("Sun 1 Nov · #1"), the "5 games · before cooldown.lol" chip and the wins and losses chip. A session mixing both is summarised from its tracked games only.
- On first launch the client usually has recent games, so History opens already filled with these neutral sessions instead of the empty state.
- Imported games from modes out of scope are not stored, and not shown (see above).
- Kept from the match history: result, champion, role (when Riot detected one), K/D/A, queue, start and length, and the stats the [[Workout formula]] needs (CS, vision score, damage to champions, and the team's kills and damage to champions for damage share and kill participation). No linked champion. The current-summoner list only describes the player, so the team totals come from the game's detail (`GET /lol-match-history/v1/games/{gameId}`): only the two team sums are kept, never another player's stats or name.
- The import runs when cooldown.lol connects to the client (retrying every 10 s until the player is logged in), and again when a game ends: on reaching the end-of-game screen, and on leaving the post-game phases, since the client's match history may list the game a little later. Until the [[Post-game workout]] is built, this is how a game played while cooldown.lol is open shows up (as *Skipped*). It stores only games not stored yet (by game id). It also completes a game stored without its stats by an earlier version when the client lists it again. A game cooldown.lol tracked itself is never replaced by its imported copy.
- The champion names and portraits come from Data Dragon, downloaded in the background ([[ADR-0004 Champion images from Data Dragon]]): until an image is there, the row shows the champion's initials.
- Built so far: both views, the neutral session headers, the *Skipped* rows with their details (cost curves and their hover) and the muted "No workout" rows. The "x of y workouts done" chip, the *Pending/Partial/Done* rows (and their START button) and the "Computing" row come with the post-game workout.
- The match history of the client is limited (recent games only, see [[#Open questions]]): older games cannot be recovered.
- Another summoner logging into the same client on the same PC: games are kept per player (`puuid`), History shows the active summoner's games only.

### Other rules
- Numbers done can only be recorded in the workout modal; History is read-only in the MVP.
- History is persisted locally and survives restarts. Nothing is sent anywhere.
- Empty state: "No game yet. Every Summoner's Rift game you finish while cooldown.lol is open is listed here."
- A game is stored once, keyed by player and game id: when its end-of-game stats are read (see [[Post-game workout]]), or when it is imported from the client's match history (see above).
- If the history file cannot be read: show the empty state with "Couldn't read your history" and keep the file untouched (never overwrite it).

## Data
- Stored per game: player (`puuid`), game id, source (*tracked* or *imported*), start and end time, queue, champion, role, linked champion, result, K/D/A, the stats used by the [[Workout formula]], the computed amounts due (and caps applied), the amounts done per exercise, and the workout status. An imported game has no linked champion; its status is *skipped*, its amounts done are 0, and its amounts due follow the current formula: computed at import, and computed again each time History's database opens, so a change of the formula reaches the games that were never offered their workout.
- Store the stats, not only the amounts: if the formula changes, tracked games keep the amounts computed at the time (amounts are stored too), and imported games are computed again from their stats.
- Location: `%APPDATA%\cooldown.lol\` on Windows, `~/Library/Application Support/cooldown.lol/` on macOS. One SQLite database, `history.sqlite` ([[ADR-0007 Local persistence]]); its `-wal` and `-shm` files next to it while the app runs.
- Sessions are computed from the stored games (pure function in `cooldown-core`), never stored.

## Design
The refresh is the `History refresh` component and its `/ Updating`, `/ Offline` and `/ Hover` variants (`History components`), placed in every title row; its tooltips are the `History refresh / Tooltip / *` frames. The chevron's tooltips are the `Game row / Expand / Tooltip / *` frames, the played time's is `Game row / Played / Tooltip`. The details of every expanded row are one `Game row / Details` component, made of `Game row / Details / Exercise` lines (cost curve; `/ Hover` shows the hover), the role badge's `Champion portrait / Tooltip / Role`. Pencil frames `History / Main / Before cooldown.lol`, `History / Main / Empty`, `History / Main / Computing`, `History / Main / No workout`, `History / Main / Games view` and `History / Main / Games view · Game expanded` (the Workouts view has no screen frame: its rows are the `Game row / Workouts / *` components), built from the `Session header / *` and `Game row / *` components and the `Stat icon / *` pictograms. See [[Screens#Frames]].

## Compliance
Only the local player's own games, after they end (live ones and the client's match history of the active summoner). Never other players' games or names, nothing during a game, no call to the Riot API.

## How to try it
`just dev` with the League client open and logged in: History fills from its match history. A throwaway data folder (`COOLDOWN_DATA_DIR=$(mktemp -d) just dev`) shows the first opening. Finish a game: History lists it once the end-of-game screen shows, or once back in the lobby. The refresh button reads the match history at once.

## Windows validation
- [ ] History survives an app restart
- [ ] A break over 1 hour starts a new session
- [ ] `history.sqlite` in `%APPDATA%\cooldown.lol\`, and gone with the folder when uninstalling with "Also delete my settings"
- [ ] Games played before the install appear as *Skipped* cards with their computed amounts, newest first, in sessions with a neutral header
- [ ] Reconnecting the client does not duplicate them
- [ ] A game played while cooldown.lol is open appears after the end-of-game screen, without restarting anything
- [ ] The refresh button shows "Updating…", then "Updated just now", and is disabled without the client
- [ ] Resting the pointer on the refresh button opens its tooltip after half a second, also when it is disabled, under the button and inside the window, and the button still takes clicks while it is shown
- [ ] How many past games the client really returns (see open questions)

## Open questions
- How many past games does the LCU match history return? Seen on the macOS client: 21 games at one moment, 3 a little later, and `begIndex`/`endIndex` ignored in the second case (the same games came back for every page). It looks like a cache the client fills as its own match history is browsed. To check on the Windows PC. The import stops when a page brings no new game, so this never loops.
- Roles: `timeline.lane` alone is not reliable (Riot called a top laner `JUNGLE`). cooldown.lol reads a `JUNGLE` without Smite (summoner spell 11) as top. To validate on more real games, on Windows too.
- Empty-state wording once the client can fill History: kept as is for now.
- Keep everything forever, or only the last N days?
- Allow fixing the numbers done from History (e.g. forgot to finish the flow)?
- Filters (role, queue) like DPM.LOL, later?
