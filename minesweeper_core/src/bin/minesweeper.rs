use minesweeper_core::get_grid;

fn main() {
    let grid = get_grid(1000, 10000, 90000);
    println!("{grid:?}");
}
