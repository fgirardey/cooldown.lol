//! Binds the [`Navigator`] to the main window (ADR-0018). The window shows the
//! screen and the modal of the stack and reports the player's moves; Rust adds
//! the Back inputs the window does not see (Esc, Alt+←, ⌘[ and the mouse back
//! button, wherever the pointer is) and types the top bar headline.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};

use slint::winit_030::{EventResult, WinitWindowAccessor, winit};
use slint::{ComponentHandle, Timer, TimerMode};

use crate::headline::{Headline, HeadlineFrame, Typing, TypingTimes};
use crate::navigator::{Modal, Move, Navigator, Route};
use crate::{AppWindow, Theme, platform};

/// How often the headline is redrawn while the cursor types: under the time of
/// one character, so that none is skipped.
const TYPING_TICK: Duration = Duration::from_millis(8);

/// The navigation of the main window. Cheap to clone.
#[derive(Clone)]
pub struct Navigation {
    inner: Rc<Inner>,
}

struct Inner {
    window: slint::Weak<AppWindow>,
    navigator: RefCell<Navigator>,
    /// The headline as drawn, so that a move can start typing from it.
    headline: RefCell<HeadlineFrame>,
    typing: Timer,
    times: TypingTimes,
    /// Windows' animation effects: off means no transition and no typing.
    animations: bool,
    /// Keyboard modifiers held, for Alt+← and ⌘[.
    modifiers: Cell<winit::keyboard::ModifiersState>,
}

impl Navigation {
    /// Shows `root` (History, or Welcome at first launch) and handles every move from then on.
    pub fn bind(window: &AppWindow, root: Route) -> Self {
        let animations = platform::animations_enabled();
        let theme = window.global::<Theme>();
        let times = TypingTimes {
            erase: millis(theme.get_motion_type_erase()),
            char: millis(theme.get_motion_type_char()),
            phase_max: millis(theme.get_motion_type_phase_max()),
            cursor_hold: millis(theme.get_motion_cursor_hold()),
        };
        window.set_animations(animations);
        let navigation = Self {
            inner: Rc::new(Inner {
                window: window.as_weak(),
                navigator: RefCell::new(Navigator::new(root)),
                headline: RefCell::new(headline_of(root).frame()),
                typing: Timer::default(),
                times,
                animations,
                modifiers: Cell::default(),
            }),
        };
        navigation.show(false);

        let open = navigation.clone();
        window.on_open_screen(move |screen| open.change(|n| n.open(route(screen))));
        let push = navigation.clone();
        window.on_push_screen(move |screen| push.change(|n| n.push(route(screen))));
        let present = navigation.clone();
        window.on_present(move |modal| {
            if let Some(modal) = modal_of(modal) {
                present.change(|n| n.present(modal));
            }
        });
        let back = navigation.clone();
        window.on_back(move || {
            back.back();
        });
        let input = navigation.clone();
        window
            .window()
            .on_winit_window_event(move |_, event| input.on_window_event(event));
        navigation
    }

    /// Replaces the stack with `route` alone ("Let's go" on the welcome).
    pub fn reset(&self, route: Route) {
        self.change(|n| {
            n.reset(route);
            true
        });
    }

    /// Back, from any input: closes the modal, else returns to the screen below.
    fn back(&self) -> bool {
        let moved = self.inner.navigator.borrow_mut().back();
        if moved {
            self.after_move();
        }
        moved
    }

    fn change(&self, apply: impl FnOnce(&mut Navigator) -> bool) {
        if apply(&mut self.inner.navigator.borrow_mut()) {
            self.after_move();
        }
    }

    fn after_move(&self) {
        let navigator = self.inner.navigator.borrow();
        tracing::debug!(
            screen = ?navigator.current(),
            modal = ?navigator.modal(),
            last_move = ?navigator.last_move(),
            "navigation"
        );
        drop(navigator);
        self.show(true);
    }

    /// Puts the navigator's state on the window. `animate`: a move just happened.
    fn show(&self, animate: bool) {
        let Some(window) = self.inner.window.upgrade() else {
            return;
        };
        let navigator = self.inner.navigator.borrow();
        let current = navigator.current();
        let screen_changed = route(window.get_screen()) != current;
        window.set_back_title(navigator.below().map_or("", Route::title).into());
        window.set_screen_title(
            match current {
                Route::Welcome | Route::History => "",
                route => route.title(),
            }
            .into(),
        );
        window.set_modal(match navigator.modal() {
            None => crate::Modal::None,
            Some(Modal::UninstallConfirm) => crate::Modal::UninstallConfirm,
        });
        // A pushed screen starts at the top; one found again under a closed one stays as it was.
        if screen_changed && current == Route::Settings && navigator.last_move() == Move::Forward {
            window.set_settings_scroll_y(0.0);
        }
        let typed = animate
            && screen_changed
            && self.inner.animations
            && navigator.last_move() != Move::Reset;
        drop(navigator);
        if screen_changed || !animate {
            self.type_headline(&window, headline_of(current), typed);
        }
        window.set_screen(screen(current));
    }

    /// Erases the headline and types `to`, or shows it at once when `typed` is false.
    fn type_headline(&self, window: &AppWindow, to: Headline, typed: bool) {
        self.inner.typing.stop();
        if !typed {
            draw(&self.inner, window, to.frame());
            return;
        }
        let from = Headline::shown(&self.inner.headline.borrow());
        let typing = Typing::new(from, to, self.inner.times);
        let started = Instant::now();
        draw(&self.inner, window, typing.at(Duration::ZERO));
        let inner = Rc::downgrade(&self.inner);
        self.inner
            .typing
            .start(TimerMode::Repeated, TYPING_TICK, move || {
                tick(&inner, &typing, started.elapsed());
            });
    }

    /// Esc, Alt+←, ⌘[ and the mouse back button go back, wherever the pointer and the
    /// focus are. Everything else goes on to the window.
    fn on_window_event(&self, event: &winit::event::WindowEvent) -> EventResult {
        use winit::event::{ElementState, MouseButton, WindowEvent};
        use winit::keyboard::{Key, NamedKey};

        let wants_back = match event {
            WindowEvent::ModifiersChanged(modifiers) => {
                self.inner.modifiers.set(modifiers.state());
                false
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                let modifiers = self.inner.modifiers.get();
                match &event.logical_key {
                    Key::Named(NamedKey::Escape) => true,
                    Key::Named(NamedKey::ArrowLeft) => modifiers.alt_key(),
                    Key::Character(text) => text.as_str() == "[" && modifiers.super_key(),
                    _ => false,
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Back,
                ..
            } => true,
            _ => false,
        };
        if wants_back && self.back() {
            EventResult::PreventDefault
        } else {
            EventResult::Propagate
        }
    }
}

/// One redraw of the headline being typed; stops once the cursor is gone.
fn tick(inner: &Weak<Inner>, typing: &Typing, elapsed: Duration) {
    let Some(inner) = inner.upgrade() else {
        return;
    };
    let Some(window) = inner.window.upgrade() else {
        return;
    };
    draw(&inner, &window, typing.at(elapsed));
    if elapsed >= typing.duration() {
        inner.typing.stop();
    }
}

fn draw(inner: &Inner, window: &AppWindow, frame: HeadlineFrame) {
    window.set_headline_main(frame.main.as_str().into());
    window.set_headline_accent(frame.accent.as_str().into());
    window.set_headline_cursor(frame.cursor);
    *inner.headline.borrow_mut() = frame;
}

/// The wordmark on History (and on the welcome, which has no top bar), the title elsewhere.
fn headline_of(route: Route) -> Headline {
    match route {
        Route::Welcome | Route::History => Headline::wordmark(),
        route => Headline::title(route.title()),
    }
}

fn screen(route: Route) -> crate::Screen {
    match route {
        Route::Welcome => crate::Screen::Welcome,
        Route::History => crate::Screen::History,
        Route::Settings => crate::Screen::Settings,
        Route::About => crate::Screen::About,
        Route::Debug => crate::Screen::Debug,
    }
}

fn route(screen: crate::Screen) -> Route {
    match screen {
        crate::Screen::Welcome => Route::Welcome,
        crate::Screen::History => Route::History,
        crate::Screen::Settings => Route::Settings,
        crate::Screen::About => Route::About,
        crate::Screen::Debug => Route::Debug,
    }
}

fn modal_of(modal: crate::Modal) -> Option<Modal> {
    match modal {
        crate::Modal::None => None,
        crate::Modal::UninstallConfirm => Some(Modal::UninstallConfirm),
    }
}

/// A duration token of `Theme`, which Slint hands over in milliseconds.
fn millis(value: i64) -> Duration {
    Duration::from_millis(u64::try_from(value).unwrap_or(0))
}
