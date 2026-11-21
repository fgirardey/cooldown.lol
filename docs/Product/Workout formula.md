---
status: draft
---

# Workout formula

How a finished game becomes a workout. Used by [[Post-game workout]]. Implemented as a pure function in `cooldown-core` (game stats + role → workout), so every rule below gets a unit test.

All numbers are **starting defaults to tune after real games**, not final values.

## Principle
Penalty only (see [[Vision]]): each exercise pays for one thing the player gave away. Reaching the target waives that exercise (zero), never a negative amount.

Each role has one **main stat**, the thing that role is judged on first. It drives the most complete exercise.

| Exercise | Unit | Pays for |
|---|---|---|
| Burpees | reps | The role's **main stat** under target |
| Squats | reps | The role's **secondary stat** under target |
| Plank | seconds | Deaths (every role) |

## Inputs
From the end-of-game stats of the local player (see [[Post-game workout#Data]]), or, for a game imported from the client's match history, from that history (see [[History#Games from before cooldown.lol]]):
- `minutes` = game length in seconds / 60
- `deaths`
- `cs` = minions killed + neutral minions killed
- `damage_share` = player damage to champions / team damage to champions
- `kill_participation` = (kills + assists) / team kills (0 if the team has no kill)
- `vision_score`

## Role stats and benchmarks
Each judged stat has three **benchmarks**, set from what is usual for the role (coaching guides and public stats; there is no official Riot reference):
- **floor**: clearly under what the role expects (about Iron/Bronze). At the floor or worse, the exercise is at its **max**.
- **average**: the typical ranked player (about Silver/Gold). At the average, the exercise is at its **mid** amount.
- **target**: what good players reach (about Platinum/Diamond and up). At the target or better, the exercise is **waived** (0).

| Role | Main stat (burpees) | Floor / average / target | Secondary stat (squats) | Floor / average / target |
|---|---|---|---|---|
| Top | CS / min | 5.0 / 6.5 / 8.0 | damage share | 15 / 21 / 27 % |
| Jungle | kill participation | 45 / 58 / 70 % | CS / min | 4.0 / 5.0 / 6.5 |
| Mid | damage share | 18 / 25 / 31 % | CS / min | 5.5 / 7.0 / 8.5 |
| Bottom | CS / min | 5.5 / 7.0 / 9.0 | damage share | 20 / 26 / 32 % |
| Support | vision score / min | 1.2 / 1.8 / 2.5 | kill participation | 45 / 58 / 70 % |

Deaths (plank, every role): floor 9, average 6, target 3. Fewer is better, so the scale runs the other way.

Sources for the starting values: CS per minute ([High Ground Gaming](https://www.highgroundgaming.com/good-cs-per-minute-lol/), [Agatasmurf](https://agatasmurf.com/lol-cs-calculator/)), vision score per minute ([LoL Theory](https://blog.loltheory.gg/vision-score-lol/)). Damage share and kill participation follow the usual split between the five roles and are the least certain.

## Amounts
| Exercise | Max (at the floor) | Mid (at the average) |
|---|---|---|
| Burpees | 15 reps | 7 reps |
| Squats | 30 reps | 15 reps |
| Plank | 90 s | 45 s |

The max is low on purpose: a workout follows every game of a session.

## Rules
1. Each exercise's amount comes from its stat's value (per minute for CS and vision, a share for damage and kill participation, a count for deaths), not from the game's length:
   - at the floor or worse: the max;
   - between the floor and the average: from the max down to the mid, in a straight line;
   - between the average and the target: from the mid down to 0, in a straight line;
   - at the target or better: 0, the exercise is waived.
2. Every amount is rounded up.
3. When the max applies (stat at the floor or worse), the UI says so.
4. **No workout** when the game is a remake (shorter than 5 minutes, or flagged as a remake by the client).

## Gap to target
Shown next to the stat so the player knows how far the waiver was, never used in the amount:

| Stat | Gap |
|---|---|
| CS / min | missed CS = CS target × `minutes` − `cs` |
| vision score / min | missed vision = vision target × `minutes` − `vision_score` |
| damage share, kill participation | percentage points under target |
| deaths | deaths over target |

## Worked example
Mid, 7/6/4, 168 CS, 22 % damage share, 28:12 (28.2 minutes):

| Exercise | Computation | Amount |
|---|---|---|
| Burpees | damage share 22 %, between floor 18 % and average 25 %: 15 − (22 − 18) / (25 − 18) × (15 − 7) = 10.4 | 11 |
| Squats | 168 / 28.2 = 6.0 CS / min, between floor 5.5 and average 7.0: 30 − (6.0 − 5.5) / (7.0 − 5.5) × (30 − 15) = 25.4 | 26 |
| Plank | 6 deaths, the average | 45 s |

## Open questions
- Should targets depend on rank or be set by the player?
- A difficulty multiplier setting (×0.5 / ×1 / ×1.5)? Needs settings persistence ([[Roadmap]]).
- Win modifier (see [[Vision#Open questions]]).
