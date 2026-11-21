//! The workout formula: how a finished game becomes burpees, squats and plank seconds
//! (`docs/Product/Workout formula.md`). Pure: game stats and role in, amounts due out.
//!
//! Each exercise pays for one stat, judged against three benchmarks of the role (floor, average,
//! target): the max at the floor or worse, the mid amount at the average, nothing from the
//! target on, in straight lines between them.

use crate::game::{Game, GameResult, PlayerStats, Role, queue_in_scope};

/// What a game costs: one amount per exercise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Workout {
    pub burpees: Amount,
    pub squats: Amount,
    /// Seconds.
    pub plank: Amount,
}

/// An amount due for one exercise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Amount {
    pub due: u32,
    /// The stat was at its floor or worse: the exercise is at its max, and the UI says so.
    pub capped: bool,
}

/// The workout of `game`, or `None` when it has none: a remake, a queue out of scope, an unknown
/// role or unreadable stats.
#[must_use]
pub fn compute(game: &Game) -> Option<Workout> {
    if game.result == GameResult::Remake
        || game.duration_secs < REMAKE_SECS
        || !queue_in_scope(game.queue_id)
    {
        return None;
    }
    let role = game.role?;
    let stats = game.stats?;

    let (main, secondary) = role_stats(role);
    Some(Workout {
        burpees: main.amount(game, &stats, BURPEES),
        squats: secondary.amount(game, &stats, SQUATS),
        plank: DEATHS.amount(game, &stats, PLANK),
    })
}

/// A game shorter than this is a remake whatever the client says.
const REMAKE_SECS: u32 = 5 * 60;

/// How much of an exercise a game can cost: `max` at the floor or worse, `mid` at the average.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scale {
    pub max: u32,
    pub mid: u32,
}

/// Burpees, paid by the role's main stat.
pub const BURPEES: Scale = Scale { max: 15, mid: 7 };
/// Squats, paid by the role's secondary stat.
pub const SQUATS: Scale = Scale { max: 30, mid: 15 };
/// Plank seconds, paid by the deaths.
pub const PLANK: Scale = Scale { max: 90, mid: 45 };

/// The plank's stat, for every role: fewer deaths is better.
pub const DEATHS: JudgedStat = JudgedStat {
    kind: StatKind::Deaths,
    benchmarks: Benchmarks {
        floor: 9.0,
        average: 6.0,
        target: 3.0,
    },
};

/// A stat the formula can judge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatKind {
    /// Creep score (minions plus monsters) per minute.
    CsPerMinute,
    /// Vision score per minute.
    VisionPerMinute,
    /// Share of the team's damage to champions, in percent.
    DamageShare,
    /// Kills plus assists over the team's kills, in percent.
    KillParticipation,
    /// Deaths in the game: the only stat where fewer is better.
    Deaths,
}

/// The three values a stat is judged against, in the stat's unit. The target is better than
/// the average, itself better than the floor: higher for most stats, lower for the deaths.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Benchmarks {
    /// Clearly under what the role expects: the exercise is at its max.
    pub floor: f64,
    /// The typical ranked player: the exercise is at its mid amount.
    pub average: f64,
    /// What good players reach: the exercise is waived.
    pub target: f64,
}

/// Where a value stands against its benchmarks, for its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    /// At the floor or worse.
    Floor,
    /// Better than the floor, worse than the average.
    BelowAverage,
    /// The average or better.
    Average,
}

/// A stat the formula judges, with the benchmarks of the role.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JudgedStat {
    pub kind: StatKind,
    pub benchmarks: Benchmarks,
}

/// The (main, secondary) stats of a role: burpees pay for the first, squats for the second.
#[must_use]
pub fn role_stats(role: Role) -> (JudgedStat, JudgedStat) {
    use StatKind::{CsPerMinute, DamageShare, KillParticipation, VisionPerMinute};
    let judged = |kind, floor, average, target| JudgedStat {
        kind,
        benchmarks: Benchmarks {
            floor,
            average,
            target,
        },
    };
    match role {
        Role::Top => (
            judged(CsPerMinute, 5.0, 6.5, 8.0),
            judged(DamageShare, 15.0, 21.0, 27.0),
        ),
        Role::Jungle => (
            judged(KillParticipation, 45.0, 58.0, 70.0),
            judged(CsPerMinute, 4.0, 5.0, 6.5),
        ),
        Role::Mid => (
            judged(DamageShare, 18.0, 25.0, 31.0),
            judged(CsPerMinute, 5.5, 7.0, 8.5),
        ),
        Role::Bottom => (
            judged(CsPerMinute, 5.5, 7.0, 9.0),
            judged(DamageShare, 20.0, 26.0, 32.0),
        ),
        Role::Support => (
            judged(VisionPerMinute, 1.2, 1.8, 2.5),
            judged(KillParticipation, 45.0, 58.0, 70.0),
        ),
    }
}

impl StatKind {
    /// The player's value in `game`: per minute, in percent, or a count for the deaths.
    #[must_use]
    pub fn value(self, game: &Game, stats: &PlayerStats) -> f64 {
        let minutes = minutes(game);
        match self {
            Self::CsPerMinute => per_minute(stats.cs, minutes),
            Self::VisionPerMinute => per_minute(stats.vision_score, minutes),
            Self::DamageShare => percent(stats.damage_to_champions, stats.team_damage_to_champions),
            Self::KillParticipation => percent(game.kills + game.assists, stats.team_kills),
            Self::Deaths => f64::from(game.deaths),
        }
    }
}

impl Benchmarks {
    /// Where `value` stands from the floor (0) to the target (1), whichever way the stat goes:
    /// below 0 under the floor, above 1 past the target.
    #[must_use]
    pub fn progress(self, value: f64) -> f64 {
        (value - self.floor) / (self.target - self.floor)
    }

    /// Where the average stands from the floor (0) to the target (1).
    #[must_use]
    pub fn average_progress(self) -> f64 {
        self.progress(self.average)
    }

    #[must_use]
    pub fn zone(self, value: f64) -> Zone {
        let progress = self.progress(value);
        if progress <= EPSILON {
            Zone::Floor
        } else if progress < self.average_progress() - EPSILON {
            Zone::BelowAverage
        } else {
            Zone::Average
        }
    }

    /// The exercise owed for `value`, before rounding: `scale.max` at the floor or worse, down to
    /// `scale.mid` at the average, down to 0 at the target and past it.
    #[must_use]
    pub fn amount(self, value: f64, scale: Scale) -> f64 {
        let (max, mid) = (f64::from(scale.max), f64::from(scale.mid));
        let progress = self.progress(value);
        let average = self.average_progress();
        if progress <= 0.0 {
            max
        } else if progress < average {
            max - progress / average * (max - mid)
        } else if progress < 1.0 {
            mid - (progress - average) / (1.0 - average) * mid
        } else {
            0.0
        }
    }
}

impl JudgedStat {
    /// The amount `game` owes for this stat, rounded up.
    #[must_use]
    pub fn amount(self, game: &Game, stats: &PlayerStats, scale: Scale) -> Amount {
        let value = self.kind.value(game, stats);
        Amount {
            due: round_up(self.benchmarks.amount(value, scale)),
            capped: self.benchmarks.zone(value) == Zone::Floor,
        }
    }

    /// How far from the target the player is, never below 0 (shown, never used in the amount):
    /// missed CS or vision points for the per-minute stats (the target times the game's minutes,
    /// minus the total), percentage points for the shares, deaths over the target.
    #[must_use]
    pub fn gap(self, game: &Game, stats: &PlayerStats) -> f64 {
        let minutes = minutes(game);
        let target = self.benchmarks.target;
        let gap = match self.kind {
            StatKind::CsPerMinute => target * minutes - f64::from(stats.cs),
            StatKind::VisionPerMinute => target * minutes - f64::from(stats.vision_score),
            StatKind::DamageShare | StatKind::KillParticipation => {
                target - self.kind.value(game, stats)
            }
            StatKind::Deaths => f64::from(game.deaths) - target,
        };
        gap.max(0.0)
    }
}

/// Float arithmetic leaves dust (25 - 22 = 3.0000000000000004): comparisons and rounding allow
/// for it.
const EPSILON: f64 = 1e-9;

/// The game's length in minutes.
fn minutes(game: &Game) -> f64 {
    f64::from(game.duration_secs) / 60.0
}

/// `total` per minute, 0 for a game without length.
fn per_minute(total: u32, minutes: f64) -> f64 {
    if minutes > 0.0 {
        f64::from(total) / minutes
    } else {
        0.0
    }
}

/// `part` over `whole`, in percent. 0 when there is no whole (a team with no kill).
fn percent(part: u32, whole: u32) -> f64 {
    if whole == 0 {
        0.0
    } else {
        f64::from(part) * 100.0 / f64::from(whole)
    }
}

/// Rounds an amount up; without the allowance for dust, 6 would be 7.
#[must_use]
pub fn round_up(value: f64) -> u32 {
    let rounded_up = (value - EPSILON).ceil().max(0.0);
    // Amounts are small (90 at most): the conversion cannot truncate.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let due = rounded_up as u32;
    due
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::GameSource;

    /// 28 minutes, `kills`/`deaths`/`assists`, and the stats the worked examples need.
    fn game(role: Role, kills: u32, deaths: u32, assists: u32, stats: PlayerStats) -> Game {
        Game {
            game_id: 1,
            source: GameSource::Imported,
            started_at: 0,
            duration_secs: 28 * 60,
            queue_id: 420,
            champion_id: 1,
            role: Some(role),
            result: GameResult::Win,
            kills,
            deaths,
            assists,
            stats: Some(stats),
            workout: None,
        }
    }

    fn stats(cs: u32, vision: u32, damage: u32, team_kills: u32, team_damage: u32) -> PlayerStats {
        PlayerStats {
            cs,
            vision_score: vision,
            damage_to_champions: damage,
            team_kills,
            team_damage_to_champions: team_damage,
        }
    }

    fn amounts(workout: Workout) -> (u32, u32, u32) {
        (workout.burpees.due, workout.squats.due, workout.plank.due)
    }

    #[test]
    fn the_worked_example_of_the_spec() {
        // Mid, 7/6/4, 168 CS, 22 % damage share, 28:12: 11 burpees, 26 squats, 45 s.
        let game = Game {
            duration_secs: 28 * 60 + 12,
            ..game(Role::Mid, 7, 6, 4, stats(168, 20, 22_000, 25, 100_000))
        };
        assert_eq!(amounts(compute(&game).unwrap()), (11, 26, 45));
    }

    #[test]
    fn reaching_the_target_waives_the_exercise() {
        // Bottom: 300 CS (10.7 / min, target 9.0), 40 % damage share (target 32 %), no death.
        let game = game(Role::Bottom, 10, 0, 5, stats(300, 10, 40_000, 20, 100_000));
        assert_eq!(amounts(compute(&game).unwrap()), (0, 0, 0));
    }

    #[test]
    fn the_average_costs_the_mid_amount() {
        // Top: 6.5 CS / min (182 CS in 28 min), 21 % damage share, 6 deaths.
        let game = game(Role::Top, 3, 6, 2, stats(182, 10, 21_000, 20, 100_000));
        assert_eq!(amounts(compute(&game).unwrap()), (7, 15, 45));
    }

    #[test]
    fn between_the_average_and_the_target_the_amount_falls_to_zero() {
        // Support: vision 2.0 / min (56 in 28 min) = 2/7 of the way from 1.8 to 2.5:
        // 7 - 2/7 x 7 = 5 burpees. KP 64 % = halfway from 58 to 70: 7.5 -> 8 squats.
        // 4 deaths = 2/3 of the way from 6 to 3: 45 - 30 = 15 s.
        let game = game(Role::Support, 0, 4, 16, stats(30, 56, 5_000, 25, 100_000));
        assert_eq!(amounts(compute(&game).unwrap()), (5, 8, 15));
    }

    #[test]
    fn at_the_floor_or_worse_the_max_applies_and_says_so() {
        // Jungle: KP 9/20 = 45 %, the floor. 2.0 CS / min, under the floor. 12 deaths.
        let game = game(Role::Jungle, 4, 12, 5, stats(56, 10, 10_000, 20, 100_000));
        let workout = compute(&game).unwrap();
        let max = |due| Amount { due, capped: true };
        assert_eq!(
            (workout.burpees, workout.squats, workout.plank),
            (max(15), max(30), max(90))
        );
    }

    #[test]
    fn just_above_the_floor_is_not_capped() {
        // Jungle: KP 46 %, 1 point above the floor: 15 - 1/13 x 8 = 14.4 -> 15, not capped.
        let game = game(Role::Jungle, 10, 0, 13, stats(182, 10, 10_000, 50, 100_000));
        assert_eq!(
            compute(&game).unwrap().burpees,
            Amount {
                due: 15,
                capped: false
            }
        );
    }

    #[test]
    fn a_team_with_no_kill_gives_no_kill_participation() {
        let game = game(Role::Jungle, 0, 0, 0, stats(182, 10, 0, 0, 100_000));
        assert_eq!(
            compute(&game).unwrap().burpees,
            Amount {
                due: 15,
                capped: true
            }
        );
    }

    #[test]
    fn deaths_run_the_other_way() {
        let deaths = DEATHS.benchmarks;
        assert_eq!(deaths.zone(10.0), Zone::Floor);
        assert_eq!(deaths.zone(9.0), Zone::Floor);
        assert_eq!(deaths.zone(7.0), Zone::BelowAverage);
        assert_eq!(deaths.zone(6.0), Zone::Average);
        assert_eq!(deaths.zone(0.0), Zone::Average);
        assert!((deaths.amount(5.0, PLANK) - 30.0).abs() < 1e-9);
        assert!((deaths.amount(2.0, PLANK)).abs() < 1e-9);
    }

    #[test]
    fn zones_split_at_the_floor_and_the_average() {
        let (cs, _) = role_stats(Role::Bottom);
        let zone = |value| cs.benchmarks.zone(value);
        assert_eq!(
            [zone(5.0), zone(5.5), zone(6.0), zone(7.0), zone(9.5)],
            [
                Zone::Floor,
                Zone::Floor,
                Zone::BelowAverage,
                Zone::Average,
                Zone::Average
            ]
        );
        assert!((cs.benchmarks.average_progress() - 1.5 / 3.5).abs() < 1e-9);
    }

    #[test]
    fn a_remake_has_no_workout() {
        let remake = Game {
            result: GameResult::Remake,
            ..game(Role::Mid, 0, 0, 0, stats(0, 0, 0, 0, 0))
        };
        assert_eq!(compute(&remake), None);
    }

    #[test]
    fn a_game_under_five_minutes_has_no_workout() {
        let short = Game {
            duration_secs: 299,
            ..game(Role::Mid, 0, 0, 0, stats(0, 0, 0, 0, 0))
        };
        assert_eq!(compute(&short), None);
    }

    #[test]
    fn a_queue_out_of_scope_has_no_workout() {
        let aram = Game {
            queue_id: 450,
            ..game(Role::Mid, 0, 0, 0, stats(0, 0, 0, 0, 0))
        };
        assert_eq!(compute(&aram), None);
    }

    #[test]
    fn without_role_or_stats_there_is_no_workout() {
        let base = game(Role::Mid, 0, 0, 0, stats(0, 0, 0, 0, 0));
        assert_eq!(
            compute(&Game {
                role: None,
                ..base.clone()
            }),
            None
        );
        assert_eq!(
            compute(&Game {
                stats: None,
                ..base
            }),
            None
        );
    }

    #[test]
    fn values_are_per_minute_in_percent_or_a_count() {
        // 28 minutes, 168 CS, 14 vision, 22 % of the damage, 11 of 25 kills, 6 deaths.
        let game = game(Role::Mid, 7, 6, 4, stats(168, 14, 22_000, 25, 100_000));
        let stats = game.stats.unwrap();
        let value = |kind: StatKind| kind.value(&game, &stats);
        assert!((value(StatKind::CsPerMinute) - 6.0).abs() < 1e-9);
        assert!((value(StatKind::VisionPerMinute) - 0.5).abs() < 1e-9);
        assert!((value(StatKind::DamageShare) - 22.0).abs() < 1e-9);
        assert!((value(StatKind::KillParticipation) - 44.0).abs() < 1e-9);
        assert!((value(StatKind::Deaths) - 6.0).abs() < 1e-9);
    }

    #[test]
    fn the_gap_is_to_the_target_and_never_negative() {
        let game = game(Role::Mid, 7, 6, 4, stats(168, 14, 22_000, 25, 100_000));
        let stats = game.stats.unwrap();
        let (main, secondary) = role_stats(Role::Mid);
        // Damage share: 31 - 22 = 9 points. CS: 8.5 x 28 - 168 = 70 CS. Deaths: 6 - 3 = 3.
        assert!((main.gap(&game, &stats) - 9.0).abs() < 1e-9);
        assert!((secondary.gap(&game, &stats) - 70.0).abs() < 1e-9);
        assert!((DEATHS.gap(&game, &stats) - 3.0).abs() < 1e-9);
        let zero_deaths = Game { deaths: 0, ..game };
        assert_eq!(DEATHS.gap(&zero_deaths, &stats), 0.0);
    }
}
