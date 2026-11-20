//! The navigation stack of the main window (ADR-0018): which screen is shown,
//! the screens under it, and the modal over it. Pure: [`crate::navigation`]
//! applies it to the window.

/// The screens from the root (History, or Welcome at first launch) to the one
/// shown, and the modal over them. Never empty.
#[derive(Debug)]
pub struct Navigator {
    stack: Vec<Route>,
    modal: Option<Modal>,
    last_move: Move,
}

impl Navigator {
    pub fn new(root: Route) -> Self {
        Self {
            stack: vec![root],
            modal: None,
            last_move: Move::Reset,
        }
    }

    /// The screen shown.
    pub fn current(&self) -> Route {
        *self.stack.last().expect("the stack is never empty")
    }

    /// The screen Back returns to, if any.
    pub fn below(&self) -> Option<Route> {
        self.stack
            .len()
            .checked_sub(2)
            .map(|index| self.stack[index])
    }

    pub fn modal(&self) -> Option<Modal> {
        self.modal
    }

    pub fn last_move(&self) -> Move {
        self.last_move
    }

    /// A top bar button: back to `route` if it is in the stack, else back to
    /// the root, then `route` pushed. So the stack stays the path to the screen,
    /// and Settings ↔ About never piles up. Returns whether anything changed.
    pub fn open(&mut self, route: Route) -> bool {
        if self.current() == route {
            return false;
        }
        if let Some(index) = self.stack.iter().position(|r| *r == route) {
            self.stack.truncate(index + 1);
            self.last_move = Move::Backward;
        } else {
            self.stack.truncate(1);
            self.stack.push(route);
            self.last_move = Move::Forward;
        }
        true
    }

    /// A link inside a screen: `route` on top of the current one.
    pub fn push(&mut self, route: Route) -> bool {
        if self.current() == route {
            return false;
        }
        self.stack.push(route);
        self.last_move = Move::Forward;
        true
    }

    /// Back: closes the modal if there is one, else returns to the screen
    /// below. Does nothing on the root: it never closes the window.
    pub fn back(&mut self) -> bool {
        if self.modal.take().is_some() {
            return true;
        }
        if self.stack.len() < 2 {
            return false;
        }
        self.stack.pop();
        self.last_move = Move::Backward;
        true
    }

    /// Replaces the stack with `route` alone, closing any modal.
    pub fn reset(&mut self, route: Route) {
        self.stack = vec![route];
        self.modal = None;
        self.last_move = Move::Reset;
    }

    /// Opens `modal` over the current screen; never over another modal.
    pub fn present(&mut self, modal: Modal) -> bool {
        if self.modal.is_some() {
            return false;
        }
        self.modal = Some(modal);
        true
    }
}

/// A screen of the main window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Welcome,
    History,
    Settings,
    About,
    Debug,
}

impl Route {
    /// The screen's name, as the back tooltip and the window title say it.
    pub fn title(self) -> &'static str {
        match self {
            Route::Welcome => "Welcome",
            Route::History => "History",
            Route::Settings => "Settings",
            Route::About => "About",
            Route::Debug => "Debug events",
        }
    }
}

/// A modal over the current screen, outside the stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modal {
    UninstallConfirm,
}

/// Which way the last move went, for its transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    /// The stack was replaced (first launch, "Let's go"): no transition.
    Reset,
    Forward,
    Backward,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(navigator: &Navigator) -> Vec<Route> {
        navigator.stack.clone()
    }

    #[test]
    fn opens_on_its_root_with_nothing_to_go_back_to() {
        let navigator = Navigator::new(Route::History);
        assert_eq!(navigator.current(), Route::History);
        assert_eq!(navigator.below(), None);
        assert_eq!(navigator.modal(), None);
        assert_eq!(navigator.last_move(), Move::Reset);
    }

    #[test]
    fn a_top_bar_button_pushes_its_screen_over_history() {
        let mut navigator = Navigator::new(Route::History);
        assert!(navigator.open(Route::Settings));
        assert_eq!(stack(&navigator), [Route::History, Route::Settings]);
        assert_eq!(navigator.below(), Some(Route::History));
        assert_eq!(navigator.last_move(), Move::Forward);
    }

    #[test]
    fn going_from_settings_to_about_does_not_pile_up() {
        let mut navigator = Navigator::new(Route::History);
        navigator.open(Route::Settings);
        navigator.open(Route::About);
        navigator.open(Route::Settings);
        assert_eq!(stack(&navigator), [Route::History, Route::Settings]);
        assert_eq!(navigator.last_move(), Move::Forward);
    }

    #[test]
    fn the_button_of_the_screen_shown_does_nothing() {
        let mut navigator = Navigator::new(Route::History);
        navigator.open(Route::About);
        assert!(!navigator.open(Route::About));
        assert_eq!(stack(&navigator), [Route::History, Route::About]);
    }

    #[test]
    fn a_top_bar_button_pops_back_to_its_screen_when_it_is_in_the_stack() {
        let mut navigator = Navigator::new(Route::History);
        navigator.open(Route::Settings);
        navigator.push(Route::Debug);
        assert!(navigator.open(Route::Settings));
        assert_eq!(stack(&navigator), [Route::History, Route::Settings]);
        assert_eq!(navigator.last_move(), Move::Backward);
    }

    #[test]
    fn a_link_pushes_over_the_current_screen() {
        let mut navigator = Navigator::new(Route::History);
        navigator.open(Route::Settings);
        assert!(navigator.push(Route::Debug));
        assert_eq!(
            stack(&navigator),
            [Route::History, Route::Settings, Route::Debug]
        );
        assert_eq!(navigator.below(), Some(Route::Settings));
    }

    #[test]
    fn back_returns_to_the_screen_below() {
        let mut navigator = Navigator::new(Route::History);
        navigator.open(Route::Settings);
        navigator.push(Route::Debug);
        assert!(navigator.back());
        assert_eq!(navigator.current(), Route::Settings);
        assert_eq!(navigator.last_move(), Move::Backward);
        assert!(navigator.back());
        assert_eq!(navigator.current(), Route::History);
    }

    #[test]
    fn back_on_the_root_does_nothing() {
        let mut navigator = Navigator::new(Route::History);
        assert!(!navigator.back());
        assert_eq!(stack(&navigator), [Route::History]);
    }

    #[test]
    fn back_closes_the_modal_before_leaving_the_screen() {
        let mut navigator = Navigator::new(Route::History);
        navigator.open(Route::Settings);
        assert!(navigator.present(Modal::UninstallConfirm));
        assert!(navigator.back());
        assert_eq!(navigator.modal(), None);
        assert_eq!(navigator.current(), Route::Settings);
    }

    #[test]
    fn a_modal_never_opens_over_another() {
        let mut navigator = Navigator::new(Route::History);
        assert!(navigator.present(Modal::UninstallConfirm));
        assert!(!navigator.present(Modal::UninstallConfirm));
    }

    #[test]
    fn lets_go_replaces_the_welcome_with_history() {
        let mut navigator = Navigator::new(Route::Welcome);
        navigator.reset(Route::History);
        assert_eq!(stack(&navigator), [Route::History]);
        assert_eq!(navigator.last_move(), Move::Reset);
        assert!(!navigator.back());
    }
}
