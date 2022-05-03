<h1 align="center">
  <br>
  <img src="https://raw.githubusercontent.com/pando85/minesweeper/master/assets/logo.svg" alt="logo" width="200">
  <br>
  Minesweeper
  <br>
  <br>
</h1>

![Build status](https://img.shields.io/github/workflow/status/pando85/minesweeper/Rust/master)
![Timer license](https://img.shields.io/github/license/pando85/timer)

Random minesweeper one node generator blazing fast.

## Benchmarking

```bash
# one execution
cargo flamegraph --bin minesweeper
# benchmarking
cargo flamegraph --bench main -- --bench
```
