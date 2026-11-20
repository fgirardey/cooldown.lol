//! The headline of the top bar, and the cursor that erases it and types the
//! next one on every move between screens (ADR-0018, frame
//! `App shell / Top bar / Headline transition`). Pure: times in, text out.

use std::time::Duration;

/// A cursor erasing one headline, then typing another.
#[derive(Clone, Debug)]
pub struct Typing {
    from: Headline,
    to: Headline,
    erase_step: Duration,
    type_step: Duration,
    cursor_hold: Duration,
}

impl Typing {
    pub fn new(from: Headline, to: Headline, times: TypingTimes) -> Self {
        let erase_step = step(times.erase, times.phase_max, from.len());
        let type_step = step(times.char, times.phase_max, to.len());
        Self {
            from,
            to,
            erase_step,
            type_step,
            cursor_hold: times.cursor_hold,
        }
    }

    /// What the headline shows `elapsed` after the move.
    pub fn at(&self, elapsed: Duration) -> HeadlineFrame {
        let erase = self.erase_step * count(self.from.len());
        let typing = self.type_step * count(self.to.len());
        if elapsed < erase {
            let erased = steps(elapsed, self.erase_step);
            return self
                .from
                .prefix(self.from.len().saturating_sub(erased), true);
        }
        if elapsed < erase + typing {
            return self
                .to
                .prefix(steps(elapsed.saturating_sub(erase), self.type_step), true);
        }
        let cursor = elapsed < erase + typing + self.cursor_hold;
        self.to.prefix(self.to.len(), cursor)
    }

    /// When the cursor is gone and the headline is final.
    pub fn duration(&self) -> Duration {
        self.erase_step * count(self.from.len())
            + self.type_step * count(self.to.len())
            + self.cursor_hold
    }
}

/// A headline: `text`, in the accent colour from character `accent_from` on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Headline {
    text: String,
    accent_from: usize,
}

impl Headline {
    /// History: "COOLDOWN" and ".LOL" in the accent colour.
    pub fn wordmark() -> Self {
        Self {
            text: "COOLDOWN.LOL".to_owned(),
            accent_from: "COOLDOWN".len(),
        }
    }

    /// A pushed screen: its title, upper case.
    pub fn title(title: &str) -> Self {
        let text = title.to_uppercase();
        let accent_from = text.chars().count();
        Self { text, accent_from }
    }

    /// The headline as a frame shows it, so a move can start from a headline
    /// still being typed.
    pub fn shown(frame: &HeadlineFrame) -> Self {
        Self {
            text: format!("{}{}", frame.main, frame.accent),
            accent_from: frame.main.chars().count(),
        }
    }

    /// The whole headline, without the cursor.
    pub fn frame(&self) -> HeadlineFrame {
        self.prefix(self.len(), false)
    }

    fn len(&self) -> usize {
        self.text.chars().count()
    }

    /// The first `chars` characters, split into their two colours.
    fn prefix(&self, chars: usize, cursor: bool) -> HeadlineFrame {
        let shown: Vec<char> = self.text.chars().take(chars).collect();
        let split = self.accent_from.min(shown.len());
        HeadlineFrame {
            main: shown[..split].iter().collect(),
            accent: shown[split..].iter().collect(),
            cursor,
        }
    }
}

/// What the top bar draws: the headline in its two colours, and the cursor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadlineFrame {
    pub main: String,
    pub accent: String,
    pub cursor: bool,
}

/// The `motion-type-*` and `motion-cursor-hold` tokens.
#[derive(Clone, Copy, Debug)]
pub struct TypingTimes {
    /// To erase one character.
    pub erase: Duration,
    /// To type one character.
    pub char: Duration,
    /// Longest erase or type phase: longer headlines go faster.
    pub phase_max: Duration,
    /// How long the cursor stays after the last character.
    pub cursor_hold: Duration,
}

/// Time per character: `per_char`, or less so that `chars` fit in `phase_max`.
fn step(per_char: Duration, phase_max: Duration, chars: usize) -> Duration {
    if chars == 0 {
        return per_char;
    }
    per_char.min(phase_max / count(chars))
}

/// Whole steps in `elapsed`.
fn steps(elapsed: Duration, step: Duration) -> usize {
    if step.is_zero() {
        return usize::MAX;
    }
    usize::try_from(elapsed.as_nanos() / step.as_nanos()).unwrap_or(usize::MAX)
}

fn count(chars: usize) -> u32 {
    u32::try_from(chars).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TIMES: TypingTimes = TypingTimes {
        erase: Duration::from_millis(10),
        char: Duration::from_millis(18),
        phase_max: Duration::from_millis(150),
        cursor_hold: Duration::from_millis(120),
    };

    fn shown(typing: &Typing, millis: u64) -> (String, String, bool) {
        let frame = typing.at(Duration::from_millis(millis));
        (frame.main, frame.accent, frame.cursor)
    }

    fn frame(main: &str, accent: &str, cursor: bool) -> (String, String, bool) {
        (main.to_owned(), accent.to_owned(), cursor)
    }

    #[test]
    fn history_to_settings_follows_the_pencil_steps() {
        let typing = Typing::new(Headline::wordmark(), Headline::title("Settings"), TIMES);
        assert_eq!(shown(&typing, 0), frame("COOLDOWN", ".LOL", true));
        assert_eq!(shown(&typing, 50), frame("COOLDOW", "", true));
        assert_eq!(shown(&typing, 100), frame("CO", "", true));
        assert_eq!(shown(&typing, 120), frame("", "", true));
        assert_eq!(shown(&typing, 192), frame("SETT", "", true));
        assert_eq!(shown(&typing, 264), frame("SETTINGS", "", true));
        assert_eq!(shown(&typing, 384), frame("SETTINGS", "", false));
        assert_eq!(typing.duration(), Duration::from_millis(384));
    }

    #[test]
    fn a_long_headline_types_faster_to_fit_the_phase() {
        let typing = Typing::new(Headline::title("Settings"), Headline::wordmark(), TIMES);
        assert_eq!(shown(&typing, 40), frame("SETT", "", true));
        assert_eq!(shown(&typing, 80), frame("", "", true));
        assert_eq!(shown(&typing, 100), frame("C", "", true));
        assert_eq!(shown(&typing, 155), frame("COOLDO", "", true));
        assert_eq!(shown(&typing, 210), frame("COOLDOWN", ".L", true));
        assert_eq!(shown(&typing, 230), frame("COOLDOWN", ".LOL", true));
        assert_eq!(shown(&typing, 350), frame("COOLDOWN", ".LOL", false));
    }

    #[test]
    fn a_move_can_start_from_a_headline_being_typed() {
        let typing = Typing::new(Headline::wordmark(), Headline::title("About"), TIMES);
        let half_erased = Headline::shown(&typing.at(Duration::from_millis(50)));
        assert_eq!(half_erased.frame(), Headline::title("Cooldow").frame());
        let back = Typing::new(half_erased, Headline::wordmark(), TIMES);
        assert_eq!(shown(&back, 0), frame("COOLDOW", "", true));
    }

    #[test]
    fn the_wordmark_keeps_its_accent_when_whole() {
        assert_eq!(
            Headline::wordmark().frame(),
            HeadlineFrame {
                main: "COOLDOWN".to_owned(),
                accent: ".LOL".to_owned(),
                cursor: false
            }
        );
    }
}
