//! What the History screen shows, from the stored games: sessions with their labels, and one
//! row per game with its stats against the role's benchmarks and the workout they turned into.
//! Plain data (no Slint types), so it is built off the UI thread and unit-tested;
//! [`SessionView::to_slint`] hands it to the window.

use std::path::PathBuf;

use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};
use cooldown_core::workout::{BURPEES, DEATHS, PLANK, SQUATS, Scale};
use cooldown_core::{
    Amount, Benchmarks, Game, GameResult, JudgedStat, NoWorkoutReason, PlayerStats, Role, Session,
    StatKind, Zone, group_into_sessions, role_stats,
};
use slint::{ComponentHandle, Image, ModelRc, VecModel};

use crate::champion_catalog::{ChampionInfo, initials};
use crate::{
    AppWindow, CostCurveData, ExerciseData, GameRowData, RoleIcons, SessionData, StatData, StatZone,
};

/// One session: its header and its games.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionView {
    pub label: String,
    pub subtitle: String,
    /// Games from before cooldown.lol only: the header has no workout progress.
    pub neutral: bool,
    pub wins: u32,
    pub losses: u32,
    pub games: Vec<GameView>,
}

/// One game row.
#[derive(Debug, Clone, PartialEq)]
pub struct GameView {
    pub champion_initials: String,
    pub champion_name: String,
    pub champion_image: Option<PathBuf>,
    pub role: Option<Role>,
    pub queue: &'static str,
    pub result: GameResult,
    /// "34:12".
    pub length: String,
    /// When the game ended: "2h ago", and "Sunday 1 November 2026, 20:35".
    pub played: String,
    pub played_at: String,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    /// "1.1 KDA".
    pub kda_ratio: String,
    /// CS per minute, damage share, kill participation, vision per minute; empty when the stats
    /// could not be read.
    pub stats: Vec<StatView>,
    pub workout: WorkoutView,
}

/// A stat of the Games view.
#[derive(Debug, Clone, PartialEq)]
pub struct StatView {
    pub value: String,
    /// The game's total: "269", "21.3k", "7/11".
    pub total: String,
    /// The plain gauge's filled part, 0 to 1.
    pub fill: f32,
    /// The role's benchmarks, when the formula judges this stat.
    pub judged: Option<JudgedView>,
}

/// A judged stat of the Games view: its zone and the benchmark gauge.
#[derive(Debug, Clone, PartialEq)]
pub struct JudgedView {
    pub zone: Zone,
    /// Where the average sits on the gauge, 0 (floor) to 1 (target).
    pub average: f32,
    pub floor_label: String,
    pub target_label: String,
}

/// The workout part of a row.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkoutView {
    /// Games from before cooldown.lol were never offered their workout: burpees, squats, plank.
    Skipped(Vec<ExerciseView>),
    /// Why there is no workout.
    NoWorkout { reason: String },
}

/// One exercise: the stat behind it, the amount due and its cost curve.
#[derive(Debug, Clone, PartialEq)]
pub struct ExerciseView {
    pub name: &'static str,
    pub stat: StatKind,
    pub value: String,
    pub zone: Zone,
    pub target: String,
    pub gap: String,
    pub due: u32,
    pub unit: &'static str,
    pub capped: bool,
    pub cap: u32,
    pub curve: CurveView,
}

/// The cost curve of an exercise (see `CostCurveData` in `ui/models.slint`).
#[derive(Debug, Clone, PartialEq)]
pub struct CurveView {
    pub benchmarks: Benchmarks,
    pub scale: Scale,
    /// Where the game stands, 0 at the floor, 1 at the target.
    pub progress: f64,
    /// The game's amount before rounding.
    pub amount: f64,
    pub floor_label: String,
    pub average_label: String,
    pub target_label: String,
}

/// The sessions of `games`, newest first, ready to show.
///
/// `local` turns a Unix time into the player's local date and time (it hides the time zone, so
/// the labels are testable); `champion` looks a champion up by id.
pub fn session_views(
    games: &[Game],
    now: i64,
    local: &impl Fn(i64) -> NaiveDateTime,
    champion: &impl Fn(u32) -> Option<ChampionInfo>,
) -> Vec<SessionView> {
    // Games of the other modes are not shown, and must not bridge two sessions either.
    let shown: Vec<Game> = games
        .iter()
        .filter(|game| game.is_in_scope())
        .cloned()
        .collect();
    let sessions = group_into_sessions(&shown);
    let today = local(now).date();
    // The rank of a session within its day counts from the first one of the day.
    let days: Vec<NaiveDate> = sessions
        .iter()
        .map(|session| local(session.started_at()).date())
        .collect();

    sessions
        .iter()
        .enumerate()
        .map(|(index, session)| {
            let day = days[index];
            // Sessions are newest first: the ones of the same day after this one are older.
            let rank = days[index + 1..]
                .iter()
                .filter(|other| **other == day)
                .count()
                + 1;
            let count = |result| {
                let games = session.games().iter();
                u32::try_from(games.filter(|game| game.result == result).count())
                    .unwrap_or(u32::MAX)
            };
            SessionView {
                label: session_label(day, today, rank),
                subtitle: session_subtitle(session),
                neutral: session.is_before_cooldown(),
                wins: count(GameResult::Win),
                losses: count(GameResult::Loss),
                games: session
                    .games()
                    .iter()
                    .map(|game| game_view(game, now, local, champion))
                    .collect(),
            }
        })
        .collect()
}

impl SessionView {
    /// The model for the window. Call on the UI thread (images and the role symbols belong to it).
    pub fn to_slint(views: &[Self], window: &AppWindow) -> ModelRc<SessionData> {
        let role_icons = window.global::<RoleIcons>();
        let sessions: Vec<SessionData> = views
            .iter()
            .map(|session| SessionData {
                label: session.label.clone().into(),
                subtitle: session.subtitle.clone().into(),
                neutral: session.neutral,
                wins: int(session.wins),
                losses: int(session.losses),
                games: model(session.games.iter().map(|game| game.to_slint(&role_icons))),
            })
            .collect();
        ModelRc::new(VecModel::from(sessions))
    }
}

impl GameView {
    fn to_slint(&self, role_icons: &RoleIcons<'_>) -> GameRowData {
        // An image that cannot be read is the same as one not downloaded yet: the initials stay.
        let image = self
            .champion_image
            .as_deref()
            .and_then(|path| Image::load_from_path(path).ok());
        let (status, reason, exercises): (&str, &str, &[ExerciseView]) = match &self.workout {
            WorkoutView::Skipped(exercises) => ("SKIPPED", "", exercises),
            WorkoutView::NoWorkout { reason } => ("NO WORKOUT", reason, &[]),
        };
        GameRowData {
            has_champion_image: image.is_some(),
            champion_image: image.unwrap_or_default(),
            champion_initials: self.champion_initials.clone().into(),
            champion_name: self.champion_name.clone().into(),
            has_role: self.role.is_some(),
            role_icon: self
                .role
                .map_or_else(Default::default, |role| role_icon(role_icons, role)),
            role_name: self.role.map_or("Role not detected", role_name).into(),
            queue: self.queue.into(),
            result_label: result_label(self.result).into(),
            win: self.result == GameResult::Win,
            remake: self.result == GameResult::Remake,
            length: self.length.clone().into(),
            played: self.played.clone().into(),
            played_at: self.played_at.clone().into(),
            kills: int(self.kills),
            deaths: int(self.deaths),
            assists: int(self.assists),
            kda_ratio: self.kda_ratio.clone().into(),
            stats: model(self.stats.iter().map(|stat| {
                StatData {
                    value: stat.value.clone().into(),
                    total: stat.total.clone().into(),
                    judged: stat.judged.is_some(),
                    zone: stat
                        .judged
                        .as_ref()
                        .map_or(StatZone::Average, |judged| zone(judged.zone)),
                    fill: stat.fill,
                    average: stat.judged.as_ref().map_or(0.0, |judged| judged.average),
                    floor_label: stat
                        .judged
                        .as_ref()
                        .map_or_else(Default::default, |judged| {
                            judged.floor_label.as_str().into()
                        }),
                    target_label: stat
                        .judged
                        .as_ref()
                        .map_or_else(Default::default, |judged| {
                            judged.target_label.as_str().into()
                        }),
                }
            })),
            status: status.into(),
            has_workout: matches!(self.workout, WorkoutView::Skipped(_)),
            reason: reason.into(),
            exercises: model(exercises.iter().map(ExerciseView::to_slint)),
        }
    }
}

impl ExerciseView {
    fn to_slint(&self) -> ExerciseData {
        let (unit_label, unit_label_one) = match self.name {
            "BURPEES" => ("burpees", "burpee"),
            "SQUATS" => ("squats", "squat"),
            _ => ("plank", "plank"),
        };
        let curve = &self.curve;
        ExerciseData {
            name: self.name.into(),
            stat: match self.stat {
                StatKind::CsPerMinute => crate::StatKind::CsPerMinute,
                StatKind::DamageShare => crate::StatKind::DamageShare,
                StatKind::KillParticipation => crate::StatKind::KillParticipation,
                StatKind::VisionPerMinute => crate::StatKind::VisionPerMinute,
                StatKind::Deaths => crate::StatKind::Deaths,
            },
            stat_label: stat_label(self.stat).into(),
            value: self.value.clone().into(),
            zone: zone(self.zone),
            target: self.target.clone().into(),
            gap: self.gap.clone().into(),
            // Games from before cooldown.lol: nothing was done.
            done: 0,
            due: int(self.due),
            unit: self.unit.into(),
            unit_label: unit_label.into(),
            unit_label_one: unit_label_one.into(),
            capped: self.capped,
            cap_label: format!("max {}{}", self.cap, self.unit).into(),
            skipped: true,
            #[allow(clippy::cast_possible_truncation)] // Small numbers, for drawing.
            curve: CostCurveData {
                average: curve.benchmarks.average_progress() as f32,
                progress: curve.progress as f32,
                max: int(curve.scale.max),
                mid: int(curve.scale.mid),
                amount: curve.amount as f32,
                floor_label: curve.floor_label.clone().into(),
                average_label: curve.average_label.clone().into(),
                target_label: curve.target_label.clone().into(),
                floor: curve.benchmarks.floor as f32,
                target: curve.benchmarks.target as f32,
                decimals: match self.stat {
                    StatKind::CsPerMinute | StatKind::VisionPerMinute => 1,
                    _ => 0,
                },
                suffix: match self.stat {
                    StatKind::DamageShare | StatKind::KillParticipation => " %",
                    _ => "",
                }
                .into(),
                noun: stat_noun(self.stat).into(),
            },
        }
    }
}

/// "TODAY · #1", "YESTERDAY · #2", "SUN 1 NOV · #1".
fn session_label(day: NaiveDate, today: NaiveDate, rank: usize) -> String {
    let name = if day == today {
        "Today".to_owned()
    } else if today.pred_opt() == Some(day) {
        "Yesterday".to_owned()
    } else {
        format!("{} {} {}", weekday(day), day.day(), month(day))
    };
    format!("{name} · #{rank}").to_uppercase()
}

fn weekday(day: NaiveDate) -> &'static str {
    &WEEKDAYS[day.weekday().num_days_from_monday() as usize][..3]
}

fn month(day: NaiveDate) -> &'static str {
    &MONTHS[day.month0() as usize][..3]
}

const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// "5 games · before cooldown.lol".
fn session_subtitle(session: &Session) -> String {
    let count = session.games().len();
    let games = if count == 1 { "game" } else { "games" };
    if session.is_before_cooldown() {
        format!("{count} {games} · before cooldown.lol")
    } else {
        format!("{count} {games}")
    }
}

fn game_view(
    game: &Game,
    now: i64,
    local: &impl Fn(i64) -> NaiveDateTime,
    champion: &impl Fn(u32) -> Option<ChampionInfo>,
) -> GameView {
    let info = champion(game.champion_id);
    let judged = game.role.map(role_stats);
    let stats = game
        .stats
        .map_or_else(Default::default, |stats| stat_views(game, &stats, judged));
    let workout = match (game.workout, game.stats, judged) {
        (Some(workout), Some(stats), Some((main, secondary))) => WorkoutView::Skipped(vec![
            exercise("BURPEES", main, BURPEES, workout.burpees, game, &stats),
            exercise("SQUATS", secondary, SQUATS, workout.squats, game, &stats),
            exercise("PLANK", DEATHS, PLANK, workout.plank, game, &stats),
        ]),
        _ => WorkoutView::NoWorkout {
            reason: no_workout_reason(game),
        },
    };
    GameView {
        champion_initials: initials(info.as_ref().map(|info| info.name.as_str())),
        champion_name: info
            .as_ref()
            .map_or_else(|| "Unknown champion".to_owned(), |info| info.name.clone()),
        champion_image: info.and_then(|info| info.image),
        role: game.role,
        queue: queue_name(game.queue_id),
        result: game.result,
        length: format!("{}:{:02}", game.duration_secs / 60, game.duration_secs % 60),
        played: played_ago(now - game.ended_at()),
        played_at: played_at(local(game.ended_at())),
        kills: game.kills,
        deaths: game.deaths,
        assists: game.assists,
        kda_ratio: format!(
            "{:.1} KDA",
            f64::from(game.kills + game.assists) / f64::from(game.deaths.max(1))
        ),
        stats,
        workout,
    }
}

/// "5m ago", "2h ago", "3d ago": how long ago a game ended, `seconds` ago.
fn played_ago(seconds: i64) -> String {
    let minutes = (seconds / 60).max(1);
    if minutes < 60 {
        format!("{minutes}m ago")
    } else if minutes < 24 * 60 {
        format!("{}h ago", minutes / 60)
    } else {
        format!("{}d ago", minutes / (24 * 60))
    }
}

/// "Sunday 1 November 2026, 20:35".
fn played_at(time: NaiveDateTime) -> String {
    let day = time.date();
    format!(
        "{} {} {} {}, {:02}:{:02}",
        WEEKDAYS[day.weekday().num_days_from_monday() as usize],
        day.day(),
        MONTHS[day.month0() as usize],
        day.year(),
        time.hour(),
        time.minute()
    )
}

/// The Games view's four stats, each against the role's benchmarks when the formula judges it.
fn stat_views(
    game: &Game,
    stats: &PlayerStats,
    judged: Option<(JudgedStat, JudgedStat)>,
) -> Vec<StatView> {
    [
        StatKind::CsPerMinute,
        StatKind::DamageShare,
        StatKind::KillParticipation,
        StatKind::VisionPerMinute,
    ]
    .into_iter()
    .map(|kind| {
        let value = kind.value(game, stats);
        let judged = judged
            .into_iter()
            .flat_map(|(main, secondary)| [main, secondary])
            .find(|judged| judged.kind == kind);
        #[allow(clippy::cast_possible_truncation)] // A fraction between 0 and 1.
        let fill = (value / gauge_scale(kind)).clamp(0.0, 1.0) as f32;
        StatView {
            value: value_label(kind, value),
            total: total_label(kind, game, stats),
            fill,
            #[allow(clippy::cast_possible_truncation)] // A fraction between 0 and 1.
            judged: judged.map(|judged| JudgedView {
                zone: judged.benchmarks.zone(value),
                average: judged.benchmarks.average_progress() as f32,
                floor_label: value_label(kind, judged.benchmarks.floor),
                target_label: value_label(kind, judged.benchmarks.target),
            }),
        }
    })
    .collect()
}

/// The exercise paid by a stat: value against the benchmarks, gap, amount and cost curve.
fn exercise(
    name: &'static str,
    judged: JudgedStat,
    scale: Scale,
    amount: Amount,
    game: &Game,
    stats: &PlayerStats,
) -> ExerciseView {
    let kind = judged.kind;
    let benchmarks = judged.benchmarks;
    let value = kind.value(game, stats);
    let gap = judged.gap(game, stats);
    let deaths = kind == StatKind::Deaths;
    ExerciseView {
        name,
        stat: kind,
        value: value_label(kind, value),
        zone: benchmarks.zone(value),
        target: format!("/ {}", value_label(kind, benchmarks.target)),
        // The plank shows no gap: the deaths over the target are the value itself.
        gap: if gap > 0.0 && !deaths {
            format!("−{} {}", gap.ceil(), gap_unit(kind))
        } else {
            String::new()
        },
        due: amount.due,
        unit: if deaths { "s" } else { "" },
        capped: amount.capped,
        cap: scale.max,
        curve: CurveView {
            benchmarks,
            scale,
            progress: benchmarks.progress(value),
            amount: benchmarks.amount(value, scale),
            floor_label: value_label(kind, benchmarks.floor),
            average_label: value_label(kind, benchmarks.average),
            target_label: value_label(kind, benchmarks.target),
        },
    }
}

fn zone(zone: Zone) -> StatZone {
    match zone {
        Zone::Floor => StatZone::Floor,
        Zone::BelowAverage => StatZone::BelowAverage,
        Zone::Average => StatZone::Average,
    }
}

/// "CS per minute", as in the stat captions of the design.
fn stat_label(kind: StatKind) -> &'static str {
    match kind {
        StatKind::CsPerMinute => "CS per minute",
        StatKind::VisionPerMinute => "Vision per minute",
        StatKind::DamageShare => "Damage share",
        StatKind::KillParticipation => "Kill participation",
        StatKind::Deaths => "Deaths",
    }
}

/// The stat in a sentence: "At 27 % damage share: …".
fn stat_noun(kind: StatKind) -> &'static str {
    match kind {
        StatKind::CsPerMinute => "CS per minute",
        StatKind::VisionPerMinute => "vision per minute",
        StatKind::DamageShare => "damage share",
        StatKind::KillParticipation => "kill participation",
        StatKind::Deaths => "deaths",
    }
}

/// "8.5" for the per-minute stats, "24 %" for the shares, "6" for the deaths.
fn value_label(kind: StatKind, value: f64) -> String {
    match kind {
        StatKind::CsPerMinute | StatKind::VisionPerMinute => format!("{value:.1}"),
        StatKind::DamageShare | StatKind::KillParticipation => format!("{value:.0} %"),
        StatKind::Deaths => format!("{value:.0}"),
    }
}

/// The game's total behind a stat: "269" CS, "21.3k" damage, "7/11" kills taken part in,
/// "22" vision score.
fn total_label(kind: StatKind, game: &Game, stats: &PlayerStats) -> String {
    match kind {
        StatKind::CsPerMinute => stats.cs.to_string(),
        StatKind::VisionPerMinute => stats.vision_score.to_string(),
        StatKind::DamageShare if stats.damage_to_champions >= 1_000 => {
            format!("{:.1}k", f64::from(stats.damage_to_champions) / 1_000.0)
        }
        StatKind::DamageShare => stats.damage_to_champions.to_string(),
        StatKind::KillParticipation => {
            format!("{}/{}", game.kills + game.assists, stats.team_kills)
        }
        StatKind::Deaths => game.deaths.to_string(),
    }
}

/// The value at which a plain gauge is full.
fn gauge_scale(kind: StatKind) -> f64 {
    match kind {
        StatKind::CsPerMinute => 10.0,
        StatKind::VisionPerMinute => 3.0,
        StatKind::DamageShare | StatKind::KillParticipation => 100.0,
        StatKind::Deaths => 20.0,
    }
}

/// The unit the gap is counted in: "CS", "vision", "pts".
fn gap_unit(kind: StatKind) -> &'static str {
    match kind {
        StatKind::CsPerMinute => "CS",
        StatKind::VisionPerMinute => "vision",
        StatKind::DamageShare | StatKind::KillParticipation => "pts",
        StatKind::Deaths => "deaths",
    }
}

/// The queues History shows (see `cooldown_core::game::queue_in_scope`).
fn queue_name(queue_id: u32) -> &'static str {
    match queue_id {
        420 => "Ranked Solo/Duo",
        440 => "Ranked Flex",
        400 => "Normal Draft",
        430 => "Normal Blind",
        480 => "Swiftplay",
        _ => "",
    }
}

fn result_label(result: GameResult) -> &'static str {
    match result {
        GameResult::Win => "WIN",
        GameResult::Loss => "LOSS",
        GameResult::Remake => "REMAKE",
    }
}

fn role_name(role: Role) -> &'static str {
    match role {
        Role::Top => "Top",
        Role::Jungle => "Jungle",
        Role::Mid => "Mid",
        Role::Bottom => "Bottom",
        Role::Support => "Support",
    }
}

/// "Remake", "Couldn't detect your role".
fn no_workout_reason(game: &Game) -> String {
    game.no_workout_reason()
        .unwrap_or(NoWorkoutReason::StatsUnreadable)
        .label()
        .to_owned()
}

fn role_icon(icons: &RoleIcons<'_>, role: Role) -> Image {
    match role {
        Role::Top => icons.get_top(),
        Role::Jungle => icons.get_jungle(),
        Role::Mid => icons.get_middle(),
        Role::Bottom => icons.get_bottom(),
        Role::Support => icons.get_utility(),
    }
}

fn int(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

fn model<T: Clone + 'static>(items: impl Iterator<Item = T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(items.collect::<Vec<_>>()))
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use cooldown_core::workout::Workout;
    use cooldown_core::{GameSource, PlayerStats};

    use super::*;

    const HOUR: i64 = 3_600;

    /// 2026-11-08 12:00 UTC, a Sunday: "now" of the tests, in a UTC "local" time.
    const NOW: i64 = 1_794_139_200;

    fn utc(seconds: i64) -> NaiveDateTime {
        DateTime::<Utc>::from_timestamp(seconds, 0)
            .unwrap()
            .naive_utc()
    }

    fn jinx(champion_id: u32) -> Option<ChampionInfo> {
        (champion_id == 222).then(|| ChampionInfo {
            name: "Jinx".to_owned(),
            image: Some(PathBuf::from("Jinx.png")),
        })
    }

    fn game(game_id: u64, started_at: i64) -> Game {
        Game {
            game_id,
            source: GameSource::Imported,
            started_at,
            duration_secs: 1_800,
            queue_id: 420,
            champion_id: 222,
            role: Some(Role::Bottom),
            result: GameResult::Win,
            kills: 9,
            deaths: 3,
            assists: 7,
            stats: Some(PlayerStats {
                cs: 200,
                vision_score: 20,
                damage_to_champions: 20_000,
                team_kills: 25,
                team_damage_to_champions: 80_000,
            }),
            workout: Some(Workout {
                burpees: Amount {
                    due: 5,
                    capped: false,
                },
                squats: Amount {
                    due: 0,
                    capped: false,
                },
                plank: Amount {
                    due: 30,
                    capped: false,
                },
            }),
        }
    }

    fn views(games: &[Game]) -> Vec<SessionView> {
        session_views(games, NOW, &utc, &jinx)
    }

    #[test]
    fn labels_the_day_and_the_rank_of_the_session_in_it() {
        // Today 02:00 and 06:00 (two sessions), yesterday 20:00, 3 November 10:00.
        let games = [
            game(4, NOW - 6 * HOUR),
            game(3, NOW - 10 * HOUR),
            game(2, NOW - 16 * HOUR),
            game(1, NOW - 5 * 24 * HOUR),
        ];
        let labels: Vec<String> = views(&games).into_iter().map(|s| s.label).collect();
        assert_eq!(
            labels,
            [
                "TODAY · #2",
                "TODAY · #1",
                "YESTERDAY · #1",
                "TUE 3 NOV · #1"
            ]
        );
    }

    #[test]
    fn the_subtitle_counts_the_games_and_says_they_are_from_before() {
        let one = views(&[game(1, NOW - HOUR)]);
        assert_eq!(one[0].subtitle, "1 game · before cooldown.lol");
        let two = views(&[game(1, NOW - 2 * HOUR), game(2, NOW - HOUR + 600)]);
        assert_eq!(two[0].subtitle, "2 games · before cooldown.lol");
        assert!(two[0].neutral);
    }

    #[test]
    fn a_row_shows_the_game() {
        let view = &views(&[game(1, NOW - HOUR)])[0].games[0];
        assert_eq!(view.champion_initials, "JI");
        assert_eq!(view.champion_name, "Jinx");
        assert_eq!(view.champion_image, Some(PathBuf::from("Jinx.png")));
        assert_eq!(view.role, Some(Role::Bottom));
        assert_eq!(view.queue, "Ranked Solo/Duo");
        assert_eq!(view.result, GameResult::Win);
        assert_eq!(view.length, "30:00");
        assert_eq!((view.kills, view.deaths, view.assists), (9, 3, 7));
        assert_eq!(view.kda_ratio, "5.3 KDA");
    }

    #[test]
    fn a_row_says_when_the_game_was_played() {
        // Started an hour ago, 30 minutes long: ended 30 minutes ago.
        let view = &views(&[game(1, NOW - HOUR)])[0].games[0];
        assert_eq!(view.played, "30m ago");
        assert_eq!(view.played_at, "Sunday 8 November 2026, 11:30");
        assert_eq!(played_ago(20), "1m ago");
        assert_eq!(played_ago(2 * HOUR + 59 * 60), "2h ago");
        assert_eq!(played_ago(3 * 24 * HOUR + HOUR), "3d ago");
    }

    #[test]
    fn judged_stats_take_their_zone_and_the_others_stay_plain() {
        // Bottom: CS per minute (5.5 / 7.0 / 9.0) pays burpees, damage share (20 / 26 / 32 %)
        // pays squats.
        let stats = &views(&[game(1, NOW - HOUR)])[0].games[0].stats;
        let values: Vec<&str> = stats.iter().map(|stat| stat.value.as_str()).collect();
        assert_eq!(values, ["6.7", "25 %", "64 %", "0.7"]);
        let totals: Vec<&str> = stats.iter().map(|stat| stat.total.as_str()).collect();
        assert_eq!(totals, ["200", "20.0k", "16/25", "20"]);
        let zones: Vec<Option<Zone>> = stats
            .iter()
            .map(|stat| stat.judged.as_ref().map(|judged| judged.zone))
            .collect();
        assert_eq!(
            zones,
            [
                Some(Zone::BelowAverage),
                Some(Zone::BelowAverage),
                None,
                None
            ]
        );
        let cs = stats[0].judged.as_ref().unwrap();
        assert_eq!(
            (cs.floor_label.as_str(), cs.target_label.as_str()),
            ("5.5", "9.0")
        );
        assert!((cs.average - 1.5 / 3.5).abs() < 1e-6);
        assert!((stats[2].fill - 0.64).abs() < 1e-6);
    }

    #[test]
    fn each_exercise_shows_its_stat_against_the_target_and_its_curve() {
        let WorkoutView::Skipped(exercises) = &views(&[game(1, NOW - HOUR)])[0].games[0].workout
        else {
            panic!("a workout");
        };
        let [burpees, squats, plank] = exercises.as_slice() else {
            panic!("three exercises");
        };
        let shown = |exercise: &ExerciseView| {
            (
                exercise.value.clone(),
                exercise.target.clone(),
                exercise.gap.clone(),
                exercise.zone,
            )
        };
        // 30 minutes: 9.0 x 30 - 200 = 70 missed CS; 32 - 25 = 7 points; no gap for the plank.
        assert_eq!(
            shown(burpees),
            (
                "6.7".into(),
                "/ 9.0".into(),
                "−70 CS".into(),
                Zone::BelowAverage
            )
        );
        assert_eq!(
            shown(squats),
            (
                "25 %".into(),
                "/ 32 %".into(),
                "−7 pts".into(),
                Zone::BelowAverage
            )
        );
        assert_eq!(
            shown(plank),
            ("3".into(), "/ 3".into(), String::new(), Zone::Average)
        );
        assert_eq!(
            (burpees.due, burpees.cap, plank.unit, plank.cap),
            (5, 15, "s", 90)
        );
        let curve = &burpees.curve;
        assert_eq!(
            (
                curve.floor_label.as_str(),
                curve.average_label.as_str(),
                curve.target_label.as_str()
            ),
            ("5.5", "7.0", "9.0")
        );
        // 6.67 CS per minute: a third of the way from the floor to the target.
        assert!((curve.progress - (200.0 / 30.0 - 5.5) / 3.5).abs() < 1e-9);
        assert!(curve.amount > 7.0 && curve.amount < 15.0);
    }

    #[test]
    fn the_session_counts_its_wins_and_losses() {
        let loss = Game {
            result: GameResult::Loss,
            ..game(2, NOW - HOUR + 600)
        };
        let session = &views(&[game(1, NOW - 2 * HOUR), loss])[0];
        assert_eq!((session.wins, session.losses), (1, 1));
    }

    #[test]
    fn an_unknown_champion_shows_a_question_mark() {
        let unknown = Game {
            champion_id: 9_999,
            ..game(1, NOW - HOUR)
        };
        let view = &views(&[unknown])[0].games[0];
        assert_eq!(view.champion_initials, "?");
        assert_eq!(view.champion_image, None);
    }

    #[test]
    fn a_game_without_a_workout_says_why() {
        let remake = Game {
            result: GameResult::Remake,
            workout: None,
            ..game(1, NOW - 3 * HOUR)
        };
        let norole = Game {
            role: None,
            workout: None,
            ..game(2, NOW - HOUR)
        };
        // Newest first, across the two sessions (the break is over an hour).
        let cards: Vec<WorkoutView> = views(&[remake, norole])
            .into_iter()
            .flat_map(|session| session.games)
            .map(|game| game.workout)
            .collect();
        let no_workout = |reason: &str| WorkoutView::NoWorkout {
            reason: reason.to_owned(),
        };
        assert_eq!(
            cards,
            [
                no_workout("Couldn't detect your role"),
                no_workout("Remake")
            ]
        );
        let remake = &views(&[Game {
            result: GameResult::Remake,
            ..game(9, NOW)
        }])[0]
            .games[0];
        assert_eq!(remake.result, GameResult::Remake);
    }

    #[test]
    fn games_of_the_other_modes_are_not_shown_and_do_not_bridge_sessions() {
        // Two ranked games 105 minutes apart, with an ARAM game that would join them: two
        // sessions, and the ARAM game is nowhere.
        let ranked = [game(1, NOW - 6 * HOUR), game(3, NOW - 6 * HOUR + 8_100)];
        let aram = Game {
            queue_id: 450,
            ..game(2, NOW - 6 * HOUR + HOUR)
        };
        let sessions = views(&[ranked[0].clone(), aram, ranked[1].clone()]);
        assert_eq!(sessions.len(), 2);
        assert!(sessions.iter().all(|session| session.games.len() == 1));
        let only_aram = views(&[Game {
            queue_id: 450,
            ..game(4, NOW - HOUR)
        }]);
        assert_eq!(only_aram, Vec::<SessionView>::new());
    }
}
