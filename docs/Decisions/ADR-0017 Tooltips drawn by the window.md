---
status: accepted
date: 2026-11-14
---

# ADR-0017 Tooltips drawn by the window

## Context
Controls explain their state in a tooltip ([[Tokens#Tooltip]]), starting with the History refresh, and other parts of the app will show details on hover (a champion portrait, for example). The tooltip follows Material Design's timing and placement: it opens 500 ms after the pointer rests on the control and stays 600 ms after it leaves; it sits centered under the control, or above it when there is no room below, and 8 px from the window edges. It must never take the pointer: the control under it keeps its clicks and hover, disabled or not.

Slint 1.18 cannot do this with a popup:
- Its built-in `Tooltip` is input-transparent, but it places itself at the pointer, and its delay and offset can't be changed.
- A `PopupWindow` built by hand is modal: while it is open, the rest of the window gets no clicks and no hover.
- The winit backend draws every popup inside the main window. A tooltip that goes past the window would need a window of its own: platform code, and content passed as data.

## Decision
- The window draws its **one tooltip** itself: `TooltipLayer` (`ui/shell/tooltip-layer.slint`), the last child of `app-window.slint`, so it is above everything, never clipped by a scroll view, and click-through (it has no `TouchArea`). The layer handles the timing and placement.
- A control asks for a tooltip by being wrapped in **`TooltipAnchor`** (`ui/components/tooltip-anchor.slint`). It is a `TouchArea` that only watches the pointer: it reports its text and its place in the window (`absolute-position`, size) to the `Tooltips` **UI global** (`ui/tooltips.slint`), which `app-window.slint` passes to the layer.
- `TooltipAnchor` writing `Tooltips` is the **one exception** to pure components (`.claude/rules/slint.md`). `Tooltips` holds what is drawn under the pointer, never app state, and only `TooltipAnchor` writes it.
- The tooltip stays **inside the window**. Its content is data: a text today (`TooltipText`), and a richer layout in `TooltipPanel` when a feature describes one.

## Alternatives considered
- **Slint's built-in `Tooltip`**: any content and no plumbing, but placed at the pointer with a fixed delay, so no Material placement and no hide delay.
- **A window of its own per tooltip** (borderless, always on top, click-through): could go past the main window, kept on the screen only. Costs platform code in `src/platform/`, its own timing from Rust, and content passed across windows. Possible later on top of this decision, if a tooltip ever needs the room.
- **Passing the hover up through callbacks** to `app-window.slint`: keeps components pure, but every screen and component between a control and the window would have to forward it.

## Consequences
- Adding a tooltip is one wrapper. The timing, placement and look are in one place.
- Moving from one control to the next while a tooltip is shown switches its text at once, without the 500 ms delay.
- A control removed while the pointer is on it sends no leave. `app-window.slint` dismisses the tooltip when the screen changes, the case that happens. A control removed in place would leave its tooltip until the next hover.
- Previews that need hover add a `TooltipLayer` bound to `Tooltips`, like the app window. Stories hold the layer open (`always-open`) since they can't hover.
