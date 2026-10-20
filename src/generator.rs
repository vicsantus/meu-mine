//! No-guess board generator, after the idea of `mines.c` from Simon Tatham's
//! Portable Puzzle Collection (MIT): place mines away from the first click, then
//! run a logical solver from that click and keep the board only if it gets solved.

use macroquad::rand::gen_range;
use std::time::{Duration, Instant};

pub fn neighbors(w: usize, h: usize, i: usize) -> impl Iterator<Item = usize> {
    let (x, y) = ((i % w) as isize, (i / w) as isize);
    (-1..=1)
        .flat_map(move |dy| (-1..=1).map(move |dx| (x + dx, y + dy)))
        .filter(move |&(nx, ny)| {
            (nx, ny) != (x, y) && nx >= 0 && ny >= 0 && nx < w as isize && ny < h as isize
        })
        .map(move |(nx, ny)| ny as usize * w + nx as usize)
}

pub fn generate(w: usize, h: usize, mines: usize, first: usize, require_unique: bool) -> Vec<bool> {
    let n = w * h;
    let (fx, fy) = (first % w, first / w);
    let mut pool: Vec<usize> = (0..n)
        .filter(|&i| (i % w).abs_diff(fx) > 1 || (i / w).abs_diff(fy) > 1)
        .collect();
    if pool.len() < mines {
        pool = (0..n).filter(|&i| i != first).collect();
    }
    assert!(pool.len() >= mines, "{mines} mines leave no safe first cell on {w}x{h}");

    // ponytail: rejection sampling instead of mines.c's mineperturb. Dense custom boards
    // rarely have a no-guess layout, so after 0.5 s this falls back to a plain random board
    // (may need a guess); port mineperturb to keep no-guess there.
    let deadline = Instant::now() + Duration::from_millis(500);
    loop {
        for k in 0..mines {
            let r = gen_range(k, pool.len());
            pool.swap(k, r);
        }
        let mut mine = vec![false; n];
        for &i in &pool[..mines] {
            mine[i] = true;
        }
        if !require_unique || Instant::now() > deadline || solve(&mine, w, h, first) {
            return mine;
        }
    }
}

/// Plays the board like a perfect logical player starting at `first`.
/// Returns true when every safe cell gets opened without guessing.
pub fn solve(mine: &[bool], w: usize, h: usize, first: usize) -> bool {
    let n = w * h;
    let total = mine.iter().filter(|&&m| m).count();
    // None = unknown, Some(false) = opened (safe), Some(true) = known mine.
    let mut known: Vec<Option<bool>> = vec![None; n];
    let mut todo = vec![first];

    loop {
        while let Some(i) = todo.pop() {
            debug_assert!(!mine[i], "solver opened a mine");
            known[i] = Some(false);
        }

        // One set per opened number: "these unknown cells hold k mines".
        let sets: Vec<Option<(Vec<usize>, i32)>> = (0..n)
            .map(|i| {
                if known[i] != Some(false) {
                    return None;
                }
                let mut cells = vec![];
                let mut k = 0;
                for j in neighbors(w, h, i) {
                    k += mine[j] as i32;
                    match known[j] {
                        None => cells.push(j),
                        Some(true) => k -= 1,
                        Some(false) => {}
                    }
                }
                (!cells.is_empty()).then_some((cells, k))
            })
            .collect();

        let mut progress = false;
        for (cells, k) in sets.iter().flatten() {
            if *k == 0 {
                progress |= mark(&mut known, &mut todo, cells.iter().copied(), false);
            } else if *k as usize == cells.len() {
                progress |= mark(&mut known, &mut todo, cells.iter().copied(), true);
            }
        }

        // Overlapping sets (origins at most 2 apart): if A has |A\B| more mines
        // than B, then A\B are all mines and B\A are all safe.
        if !progress {
            for a in 0..n {
                let Some((sa, ka)) = &sets[a] else { continue };
                let (ax, ay) = ((a % w) as isize, (a / w) as isize);
                for dy in -2..=2 {
                    for dx in -2..=2 {
                        let (bx, by) = (ax + dx, ay + dy);
                        if (dx, dy) == (0, 0) || bx < 0 || by < 0 || bx >= w as isize || by >= h as isize {
                            continue;
                        }
                        let Some((sb, kb)) = &sets[by as usize * w + bx as usize] else { continue };
                        let a_only: Vec<usize> = sa.iter().copied().filter(|c| !sb.contains(c)).collect();
                        if ka - kb == a_only.len() as i32 {
                            let b_only = sb.iter().copied().filter(|c| !sa.contains(c));
                            progress |= mark(&mut known, &mut todo, a_only, true);
                            progress |= mark(&mut known, &mut todo, b_only, false);
                        }
                    }
                }
            }
        }

        // Global mine count.
        if !progress {
            let unknown: Vec<usize> = (0..n).filter(|&i| known[i].is_none()).collect();
            let left = total - known.iter().filter(|&&k| k == Some(true)).count();
            if left == 0 || left == unknown.len() {
                progress |= mark(&mut known, &mut todo, unknown, left > 0);
            }
        }

        if !progress {
            return known.iter().filter(|&&k| k == Some(false)).count() == n - total;
        }
    }
}

fn mark(
    known: &mut [Option<bool>],
    todo: &mut Vec<usize>,
    cells: impl IntoIterator<Item = usize>,
    is_mine: bool,
) -> bool {
    let mut changed = false;
    for c in cells {
        if known[c].is_none() {
            changed = true;
            if is_mine {
                known[c] = Some(true);
            } else {
                todo.push(c);
            }
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_boards_are_solvable_from_first_click() {
        for &(w, h, m) in &[(9, 9, 10), (16, 16, 40), (30, 16, 99)] {
            let first = w * h / 2 + w / 2;
            let mine = generate(w, h, m, first, true);
            assert_eq!(mine.iter().filter(|&&x| x).count(), m);
            assert!(!mine[first]);
            assert!(neighbors(w, h, first).all(|j| !mine[j]));
            assert!(solve(&mine, w, h, first));
        }
    }

    #[test]
    fn dense_custom_boards_do_not_hang() {
        for &(w, h, m) in &[(9, 9, 72), (50, 50, 999)] {
            let mine = generate(w, h, m, 0, true);
            assert_eq!(mine.iter().filter(|&&x| x).count(), m);
            assert!(!mine[0]);
        }
    }

    #[test]
    fn solver_needs_logic_not_luck() {
        // . 1 M .  -> the 1 pins the mine; the last cell is safe by the global count.
        assert!(solve(&[false, false, true, false], 4, 1, 0));
        // . 1 M     both 1s see the same two unknowns: a true 50/50.
        // . 1 .
        assert!(!solve(&[false, false, true, false, false, false], 3, 2, 0));
    }
}
