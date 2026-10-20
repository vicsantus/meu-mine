//! Game rules: the equivalent of the original app's `MinesweeperRefereeImpl`.

use crate::generator::{generate, neighbors};
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Hidden,
    Flag,
    Question,
    Open,
}

pub struct Game {
    pub w: usize,
    pub h: usize,
    pub mines: usize,
    pub mine: Vec<bool>,
    pub adj: Vec<u8>,
    pub state: Vec<Cell>,
    pub started: bool,
    pub lost: bool,
    pub exploded: Option<usize>,
}

impl Game {
    pub fn new(w: usize, h: usize, mines: usize) -> Self {
        Game {
            w,
            h,
            mines,
            mine: vec![false; w * h],
            adj: vec![0; w * h],
            state: vec![Cell::Hidden; w * h],
            started: false,
            lost: false,
            exploded: None,
        }
    }

    pub fn won(&self) -> bool {
        self.started && !self.lost && (0..self.state.len()).all(|i| self.mine[i] || self.state[i] == Cell::Open)
    }

    pub fn over(&self) -> bool {
        self.lost || self.won()
    }

    pub fn remaining_mines(&self) -> isize {
        self.mines as isize - self.state.iter().filter(|&&c| c == Cell::Flag).count() as isize
    }

    /// EXPLORE: open one cell; the first one generates a no-guess board around it.
    pub fn explore(&mut self, i: usize) {
        if self.over() || !matches!(self.state[i], Cell::Hidden | Cell::Question) {
            return;
        }
        if !self.started {
            self.set_mines(generate(self.w, self.h, self.mines, i, true));
        }
        self.explore_from(i);
    }

    /// EXPLORE_ALL (chord): on an open number whose flags match it, open every other neighbor.
    pub fn explore_all(&mut self, i: usize) {
        if self.over() || self.state[i] != Cell::Open {
            return;
        }
        let flags = neighbors(self.w, self.h, i).filter(|&j| self.state[j] == Cell::Flag).count();
        let targets: Vec<usize> = neighbors(self.w, self.h, i)
            .filter(|&j| matches!(self.state[j], Cell::Hidden | Cell::Question))
            .collect();
        if flags != self.adj[i] as usize || targets.is_empty() {
            return;
        }
        for j in targets {
            if self.state[j] != Cell::Open {
                self.explore_from(j);
            }
        }
    }

    /// FLAG: Hidden -> Flag -> Question -> Hidden.
    pub fn toggle_flag(&mut self, i: usize) {
        if self.over() || self.state[i] == Cell::Open {
            return;
        }
        self.state[i] = match self.state[i] {
            Cell::Hidden => Cell::Flag,
            Cell::Flag => Cell::Question,
            _ => Cell::Hidden,
        };
    }

    /// Flood fill (BFS): open `i`, and keep opening through cells with no adjacent mine.
    /// On a win the remaining mines get flagged, like Mines does.
    fn explore_from(&mut self, i: usize) {
        if self.mine[i] {
            self.lost = true;
            self.exploded = Some(i);
            return;
        }
        self.state[i] = Cell::Open;
        let mut queue = VecDeque::from([i]);
        while let Some(c) = queue.pop_front() {
            if self.adj[c] != 0 {
                continue;
            }
            for j in neighbors(self.w, self.h, c) {
                if matches!(self.state[j], Cell::Hidden | Cell::Question) {
                    self.state[j] = Cell::Open;
                    queue.push_back(j);
                }
            }
        }
        if self.won() {
            for j in 0..self.mine.len() {
                if self.mine[j] {
                    self.state[j] = Cell::Flag;
                }
            }
        }
    }

    fn set_mines(&mut self, mine: Vec<bool>) {
        self.adj = (0..mine.len())
            .map(|i| neighbors(self.w, self.h, i).filter(|&j| mine[j]).count() as u8)
            .collect();
        self.mine = mine;
        self.started = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Cell::*;

    // 4x3 board, mine at (3,0):
    //   0 0 1 M
    //   0 0 1 1
    //   0 0 0 0
    fn board() -> Game {
        let mut g = Game::new(4, 3, 1);
        let mut mine = vec![false; 12];
        mine[3] = true;
        g.set_mines(mine);
        g
    }

    #[test]
    fn flood_fill_stops_at_numbers_and_wins() {
        let mut g = board();
        g.explore(0);
        assert_eq!(g.state.iter().filter(|&&c| c == Open).count(), 11);
        assert!(g.won());
        assert_eq!(g.state[3], Flag);
        assert_eq!(g.remaining_mines(), 0);
    }

    #[test]
    fn flag_blocks_flood_fill() {
        let mut g = board();
        g.toggle_flag(1);
        g.explore(0);
        assert_eq!(g.state[1], Flag);
        assert!(!g.won());
    }

    #[test]
    fn flag_cycles() {
        let mut g = board();
        for want in [Flag, Question, Hidden] {
            g.toggle_flag(5);
            assert_eq!(g.state[5], want);
        }
    }

    #[test]
    fn chord_needs_matching_flags() {
        let mut g = board();
        g.explore(2); // the "1" at (2,0)
        g.explore_all(2); // no flag yet: nothing happens
        assert_eq!(g.state[1], Hidden);

        g.toggle_flag(7); // wrong flag on (3,1)
        g.explore_all(2); // opens (3,0): boom
        assert!(g.lost);
        assert_eq!(g.exploded, Some(3));
    }
}
