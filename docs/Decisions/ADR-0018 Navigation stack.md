---
status: accepted
date: 2026-11-20
---

# ADR-0018 Navigation stack

## Context
The window has one main screen, History, and a few screens opened from it: Settings, About, and the throwaway Debug screen under Settings ([[Screens]]). Modals open over them: the uninstall confirmation, and soon the workout prompt and its steps ([[Post-game workout]]).

Navigation so far is a single `screen` property, set in eight places of `app-window.slint`, each "back" pointing to a fixed screen. This causes several problems:
- Nothing guarantees that going back returns the player to where they came from. Each new screen has to pick its own back target by hand.
- Screens are built in `if`s, so History is destroyed as soon as another screen opens. Back on History, the scroll position and the opened rows are gone.
- The only way back is a small "‹ History" link in the content. In Settings it scrolls away with the page. Esc, Alt+← and the mouse back button do nothing, although the mouse back button is common on Windows mice.
- The "Debug events" link reuses the back link (chevron ←) to go forward.
- Screens swap instantly, with no cue that the player went deeper or came back.
- Rust also has to drive navigation: the welcome at start, and going back to History when a workout is ready. With bare property writes, nothing keeps those consistent with the player's own moves.

The platform guidelines agree on the model:
- **Android, "Principles of navigation"**: a fixed start destination, the navigation state is a stack, Back never leaves the app, Up and Back do the same thing inside the app, and navigating on its own (a deep link) yields the same stack as getting there by hand.
- **Apple HIG** (Navigation, Modality): hierarchical navigation with Back at the leading edge, titled with the previous screen. Modals hold self-contained tasks, always have an obvious way to close, and are never stacked.
- **Material 3** (App bars guidelines): the app bar's leading button navigates, and to go back it is a back arrow. On a home screen, that leading element can be the product's logo instead, either decorative or returning home. The headline describes the current page, the section or the product. Forward and backward moves use a shared-axis transition. Back closes an open dialog before it leaves the screen.
- **Windows**, the target: a back button at the left of the title area (Settings, Microsoft Store), Alt+← and the mouse back button. Esc closes dialogs.

## Decision
Navigation is a **stack of screens owned by Rust**, with modals kept apart from it. The window renders it and reports the player's moves.

### Model
- **Root**: History, always at the bottom of the stack. The one exception is the first launch, where Welcome is shown alone. "Let's go" then resets the stack to `[History]`, and Welcome can't be reached by going back.
- **Pushed screens**: Settings, About, and Debug (opened from Settings). A pushed screen is built when it is pushed and dropped when it is popped. Its scroll position starts at the top each time.
- **Modals**: the uninstall confirmation, the workout prompt and the workout steps.
  - They sit over the current screen and are not part of the stack.
  - Only one is open at a time.
  - A click on the scrim never closes one. Esc and the modal's own buttons do.
- **Screens below stay alive**. History stays built and hidden under the pushed screens, so its scroll position, opened rows and Games/Workouts view are as the player left them.

### Moves
- **Open from the top bar** (Settings, About):
  - If the screen is already in the stack, the stack pops back to it. Clicking the button of the screen being shown therefore does nothing, and clicking Settings from Debug returns to Settings.
  - Otherwise the stack pops back to History, then pushes the screen.
  - The stack is therefore always the path in the screen hierarchy (`History › Settings › Debug` at most). The back arrow always returns to the screen above it in that path, and Settings ↔ About never piles up.
- **Push**: a link inside a screen ("Debug events" in Settings) pushes its screen. A link that goes forward ends with a chevron →, never ←.
- **Back** closes the open modal if there is one. Otherwise it pops the top screen, and on History it does nothing: Back never closes or hides the window. One action, several inputs:
  - the **back arrow** in the top bar;
  - **Esc**;
  - **Alt+←**, and **⌘[** on macOS for development;
  - the **mouse back button**.
- There is no forward move: Alt+→ and the mouse forward button do nothing.
- **What Esc does on a modal** is the modal's own way out:
  - uninstall confirmation: **Cancel**;
  - workout prompt: **Later**;
  - workout steps: **Close (×)**, with the rules of [[Post-game workout]] (recorded steps kept, plank paused).
- **Navigation on its own** happens only when a workout is ready.
  - The stack pops back to History from **any** screen, then the prompt opens. This is the same stack the player would get by hand.
  - If the uninstall confirmation is open, the prompt waits until it is closed.
  - On Welcome, the prompt waits for "Let's go".
  - Nothing else changes the screen on its own. Showing the window from the notification area or from a second launch keeps the screen it was on.

### Look
- **Leading slot and headline** of the top bar, as Material lays out an app bar:
  - The leading slot is 36 px wide. On History it holds the Mark, which is decorative and does nothing on click. On a pushed screen, the **back arrow** (a 36 px icon button) takes its place. The tooltip names the screen it returns to: "Back to History (Esc)", "Back to Settings (Esc)".
  - The headline is "COOLDOWN.LOL" on History, and the title of the current screen on a pushed one ("SETTINGS", "ABOUT", "DEBUG EVENTS"), in the wordmark's style.
  - On every move, a **cursor erases the headline, then types the next one**, like a game console: a block cursor in `accent`, slanted like the italic text.
    - It erases at `motion-type-erase` (10 ms) per character and types at `motion-type-char` (18 ms) per character. Each phase lasts at most `motion-type-phase-max` (150 ms): a long title types faster.
    - The cursor stays `motion-cursor-hold` (120 ms) after the last character, then goes.
    - Meanwhile, the leading slot fades through (out, swap at the midpoint, in) over the screen transition.
  - The leading slot and the headline stay where they are, and so does the rest of the bar (connection status, Settings and About buttons): **nothing in the bar moves**.
  - The back arrow never scrolls with the content. History has none: there is no disabled back button.
  - The "‹ History" links and the big titles inside the content of Settings and About are removed, since the top bar now names the screen.
- **Transitions**, shared axis (Material) in a desktop size, over `motion-medium` (200 ms), easing out:
  - a push slides the new screen in by 30 px from the right while it fades in, and the screen below fades out;
  - a pop plays the same transition in reverse;
  - a modal fades its scrim in and lifts its content by a few pixels;
  - no transition plays when Windows' **Animation effects** setting is off: no cursor either, and the headline changes at once.
- **Window title**: "cooldown.lol" on History and Welcome, "<Screen> — cooldown.lol" on pushed screens ("Settings — cooldown.lol"), for Alt+Tab and the taskbar.

### Code
- `crates/cooldown-app/src/navigator.rs`: a pure **`Navigator`**, unit-tested.
  - State: `stack: Vec<Route>`, never empty, and `modal: Option<Modal>`.
  - Moves: `open(route)` (the top bar rule above), `push(route)`, `back() -> bool` (`false` when there is nothing to close or pop), `reset(route)`, `present(modal)`.
  - Each move also records whether it went forward or backward.
- `crates/cooldown-app/src/headline.rs`: the typing, pure and unit-tested. It gives the headline (in its two colours) and the cursor at any time after a move, from the `motion-type-*` tokens, which Rust reads from `Theme`.
- `crates/cooldown-app/src/navigation.rs` binds them to the window, after every move, whether it came from the player or from Rust:
  - it sets the screen, the modal, the screen titles and the headline frames, redrawn every 8 ms while the cursor types;
  - it puts Settings back at the top when Settings is pushed;
  - it handles Esc, Alt+←, ⌘[ and the mouse back button from the window's own input events (winit). They work wherever the focus and the pointer are, with no `FocusScope`.
- `Route` and `Modal` are mirrored by Slint enums: `Screen` (`ui/screen.slint`) and `Modal` (`ui/modal.slint`).
- `app-window.slint` receives that state and reports moves through `open-screen(Screen)` (top bar), `push-screen(Screen)` (links), `present(Modal)` and `back()` (back arrow, Cancel). It no longer writes `screen` itself.
- Every screen of the shell has its own **`ScreenLayer`** (`ui/shell/screen-layer.slint`), stacked and always built. The layer shown is opaque; the others are faded out, and a pushed screen waits `motion-push-offset` to the right. So the transitions are bindings that Slint animates, with no direction to pass.
- Screens stay pure: they only emit their own callbacks (`open-debug`, `cancel`, …), which `app-window.slint` turns into moves.
- Reading Windows' Animation effects setting is platform code (`src/platform/`): `MinAnimate` in `HKCU\Control Panel\Desktop\WindowMetrics`, read once at launch. macOS returns "on".

## Alternatives considered
- **A static hierarchy in Slint**: a `parent(screen)` function, where Back means "go to the parent". This is enough while the hierarchy has no parameters, and it costs no Rust. Rust still has to drive navigation (welcome, workout ready), though, and Slint can't hold a stack of its own, since it can't push to an array. Any screen with an argument (a game, a session) would need the stack anyway.
- **Navigation state in `cooldown-core`** (`CompanionState`): which screen is shown is window state, not domain state. Core keeps the game and workout logic. The app reacts to "workout ready" by calling `pop_to_root()`.
- **Pushing on top of anything from the top bar**: the stack grows without bound (Settings › About › Settings › …), Back replays useless steps, and where the back arrow leads changes from one click to the next.
- **Keeping the back link in the content**, or making it sticky, or adding a sub-bar "‹ History · SETTINGS" under the top bar: the first two keep Back somewhere other than where every platform puts it. The sub-bar costs about 44 px of height on every pushed screen.
- **A "‹ History" button before the Mark** (iOS-style, with the destination's title): it pushes the Mark, the wordmark and the status to the right on every push, a layout shift the player sees each time.
- **The Mark as the back button** (with a chevron in the margin, or the sand flowing back up as it is clicked): no layout shift, but a logo is not recognised as Back, and Material keeps the logo for the home screen. Flipping the hourglass would also swap the sand, which [[Brand]] forbids.
- **A forward move** (Alt+→, mouse forward): neither iOS nor Android has one, and with a stack this shallow it adds state and nothing for the player.
- **Closing modals on a scrim click** (Material dialogs): a stray click would close the workout prompt or interrupt the plank. Esc is already the deliberate way out.

## Consequences
- Every new screen gets correct Back, keyboard and mouse handling, its top bar title and its transition for free. It declares its route and where it is opened from.
- Navigation can be unit-tested in Rust, including the moves triggered by the app (workout ready, welcome).
- History must no longer be built in an `if`: it stays built under the pushed screens. Its rows keep their opened state, so anything that rebuilds the sessions model must keep it too.
- `TooltipLayer` keeps dismissing the tooltip on every move ([[ADR-0017 Tooltips drawn by the window]]).
- The Back inputs depend on Slint's winit backend (`unstable-winit-030`, already used to bring the window to the front).
- The app window preview (`just preview app-window`) fakes the navigation with a one-level stack, without typing.
- The headline transition is drawn step by step in `App shell / Top bar / Headline transition` (`design/cooldown.pen`). The Settings and About frames use `App shell / Top bar / Pushed`.
- Left for the post-game workout: popping back to History and the prompt waiting behind the uninstall confirmation or the welcome, when a workout is ready.
