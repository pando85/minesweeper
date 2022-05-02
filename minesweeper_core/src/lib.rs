use std::error::Error;
use std::fmt;
use std::sync::mpsc;
use std::thread;

use rand::rngs::SmallRng;
use rand::{Rng, RngCore, SeedableRng};

fn pick_random<T>(rnd: &mut impl RngCore, v: &mut Vec<T>) -> (T, Vec<T>)
where
    T: Clone,
{
    let index: usize = rnd.gen_range(0..v.len());
    let random = v[index].clone();
    v[index] = v.last().unwrap().clone();
    v.pop();
    (random, v.to_vec())
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
        return Err(MinesOverflowError)
    }

    let mut mines = vec![false; len];
    let mut indexes: Vec<usize> = (0..len).collect();
    let mut rng = SmallRng::from_entropy();

    for _ in 0..mines_n {
        let (mine_index, new_indexes) = pick_random(&mut rng, &mut indexes);
        indexes = new_indexes;
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
    let mut workers = (0..width).collect();

    let mut threads = vec![];
    let mut remaining_mines = mines_n.clone();
    // limit maximum threads to maximum CPUs available
    // split grid in equal areas to match maximum workers
    for i in 0..width {
        let (x, new_workers) = pick_random(&mut rng, &mut workers);
        workers = new_workers;
        let worker_tx = tx.clone();
        let worker_mines = if i != width - 1 {
            let possible_mines = rng.gen_range(0..remaining_mines);
            if possible_mines > height {
                height
            } else {
                possible_mines
            }
        } else {
            remaining_mines
        };
        remaining_mines -= worker_mines;

        threads.push(thread::spawn(move || {
            let mines = get_mines(worker_mines, height).unwrap();
            worker_tx.send((x, mines)).unwrap();
        }));
    }

    let mut received = vec![];
    for _ in 0..width {
        received.push(rx.recv().unwrap());
    }

    let mut mines = vec![];
    for i in 0..received.len() {
        for r in &received {
            if i == r.0 as usize {
                mines.push(r.1.clone());
                break;
            }
        }
    }
    Ok(mines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_random() {
        let mut rng = SmallRng::from_entropy();
        let (value, v) = pick_random(&mut rng, &mut vec![1, 2, 3]);
        assert_eq!(value >= 1, true);
        assert_eq!(value <= 3, true);
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
                .fold(0, |acc, x| if x == true { acc + 1 } else { acc });
                assert_eq!(mines_n, input_mines)
            } else {
                assert_eq!(mines_vec_result, Err(MinesOverflowError))
            }
        }
    }

    #[test]
    fn test_get_grid() {
        let mut rng = SmallRng::from_entropy();
        for _ in 0..100 {
            let width = rng.gen_range(0..999);
            let height = rng.gen_range(0..999);
            let input_mines = rng.gen_range(0..998001);

            let mines_grid_res = get_grid(width, height, input_mines);
            if let Ok(mines_grid) = mines_grid_res {
                let mines_n = mines_grid
                    .into_iter()
                    .flatten()
                    .fold(0, |acc, x| if x == true { acc + 1 } else { acc });
                assert_eq!(mines_n, input_mines)
            } else {
                assert_eq!(mines_grid_res, Err(MinesOverflowError))
            }
        }
    }
}
