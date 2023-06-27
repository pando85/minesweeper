use std::error::Error;
use std::fmt;
use std::sync::mpsc;
use std::thread;

use num_integer::div_ceil;
use rand::rngs::SmallRng;
use rand::{Rng, RngCore, SeedableRng};

const MINES_THRESHOLD: usize = 10000;

fn pick_random<T>(rng: &mut impl RngCore, v: &mut Vec<T>) -> Option<T>
where
    T: Clone,
{
    if v.is_empty() {
        return None;
    }
    let index: usize = rng.gen_range(0..v.len());
    let random = v[index].clone();
    v[index] = v.last().unwrap().clone();
    v.pop();
    Some(random)
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct MinesOverflowError;

impl fmt::Display for MinesOverflowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There are more mines than possible slots")
    }
}

impl Error for MinesOverflowError {}

fn get_mines(mines_n: usize, len: usize) -> Result<Vec<bool>, MinesOverflowError> {
    if mines_n > len {
        return Err(MinesOverflowError);
    }

    let mut mines = vec![false; len];
    let mut indexes: Vec<usize> = (0..len).collect();
    let mut rng = SmallRng::from_entropy();

    for _ in 0..mines_n {
        // safe unwrap: len is already checked
        let mine_index = pick_random(&mut rng, &mut indexes).unwrap();
        mines[mine_index] = true;
    }
    Ok(mines)
}

pub fn get_grid(
    width: usize,
    height: usize,
    mines_n: usize,
) -> Result<Vec<Vec<bool>>, MinesOverflowError> {
    if width * height < mines_n {
        return Err(MinesOverflowError);
    }

    let (tx, rx) = mpsc::channel();
    let mut rng = SmallRng::from_entropy();
    let mut cols = (0..width).collect();

    if mines_n > MINES_THRESHOLD {
        let max_threads = num_cpus::get();

        let cols_per_thread = div_ceil(width, max_threads);

        let mut threads = vec![];
        let mut remaining_mines = mines_n;
        let mut col_counter = 0;

        for _ in 0..max_threads {
            let mut thread_work = vec![];
            'thread_work: for _ in 0..cols_per_thread {
                let x = match pick_random(&mut rng, &mut cols) {
                    None => break 'thread_work,
                    Some(y) => y,
                };
                let col_mines = if col_counter != width - 1 {
                    let possible_mines = rng.gen_range(0..remaining_mines);
                    if possible_mines > height {
                        height
                    } else {
                        possible_mines
                    }
                } else {
                    remaining_mines
                };
                let possible_remaining_mines = remaining_mines - col_mines;

                let remaining_cols = width - 1 - col_counter;
                let remaining_slots = remaining_cols * height;
                let col_mines = if possible_remaining_mines > remaining_slots {
                    height
                } else {
                    col_mines
                };

                col_counter += 1;
                remaining_mines -= col_mines;
                thread_work.push((x, col_mines))
            }
            let thread_tx = tx.clone();

            threads.push(thread::spawn(move || {
                let mut result = vec![];
                for work in thread_work {
                    let mines = get_mines(work.1, height).unwrap();
                    result.push((work.0, mines))
                }
                thread_tx.send(result).unwrap();
            }));
        }

        let mut received = vec![];
        for _ in 0..max_threads {
            received.push(rx.recv().unwrap());
        }

        for thread in threads {
            thread.join().unwrap();
        }

        let mut mines = vec![vec![]; width];
        for r in &received {
            for (col_idx, col_mines) in r {
                mines[*col_idx] = col_mines.clone();
            }
        }
        Ok(mines)
    } else {
        let mut grid = vec![vec![false; height]; width];
        let mut indexes: Vec<(usize, usize)> = (0..width)
            .flat_map(|x| (0..height).map(move |y| (x, y)))
            .collect();

        for _ in 0..mines_n {
            let mine_idx = pick_random(&mut rng, &mut indexes).unwrap();
            grid[mine_idx.0][mine_idx.1] = true;
        }
        Ok(grid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_random() {
        let mut rng = SmallRng::from_entropy();
        let mut v = vec![1, 2, 3];
        let value = pick_random(&mut rng, &mut v).unwrap();
        assert!(value >= 1);
        assert!(value <= 3);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_get_mines() {
        let mut rng = SmallRng::from_entropy();
        for _ in 0..100 {
            let input_mines = rng.gen_range(0..9999);
            let total_len = rng.gen_range(0..9999);
            let mines_vec_result = get_mines(input_mines, total_len);
            if let Ok(mines_vec) = mines_vec_result {
                let mines_n = mines_vec
                    .into_iter()
                    .fold(0, |acc, x| if x { acc + 1 } else { acc });
                assert_eq!(mines_n, input_mines)
            } else {
                assert_eq!(mines_vec_result, Err(MinesOverflowError))
            }
        }
    }

    #[test]
    fn test_get_grid_single_thread() {
        let mut rng = SmallRng::from_entropy();
        for _ in 0..100 {
            let width = rng.gen_range(0..(MINES_THRESHOLD as f64).sqrt() as usize);
            let height = rng.gen_range(0..(MINES_THRESHOLD as f64).sqrt() as usize);
            let input_mines = rng.gen_range(0..MINES_THRESHOLD);

            let mines_grid_res = get_grid(width, height, input_mines);
            if let Ok(mines_grid) = mines_grid_res {
                let mines_n =
                    mines_grid
                        .into_iter()
                        .flatten()
                        .fold(0, |acc, x| if x { acc + 1 } else { acc });
                assert_eq!(mines_n, input_mines)
            } else {
                assert_eq!(mines_grid_res, Err(MinesOverflowError))
            }
        }
    }

    #[test]
    fn test_get_grid_multithread() {
        let mut rng = SmallRng::from_entropy();
        for _ in 0..100 {
            let width = rng.gen_range((MINES_THRESHOLD as f64).sqrt() as usize..999);
            let height = rng.gen_range((MINES_THRESHOLD as f64).sqrt() as usize..999);
            let input_mines = rng.gen_range(MINES_THRESHOLD..99999);

            dbg!(width);
            dbg!(height);
            dbg!(input_mines);
            let mines_grid_res = get_grid(width, height, input_mines);
            if let Ok(mines_grid) = mines_grid_res {
                let mines_n =
                    mines_grid
                        .into_iter()
                        .flatten()
                        .fold(0, |acc, x| if x { acc + 1 } else { acc });
                assert_eq!(mines_n, input_mines)
            } else {
                assert_eq!(mines_grid_res, Err(MinesOverflowError))
            }
        }
    }
}
