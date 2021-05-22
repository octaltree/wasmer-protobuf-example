# wasmer-protobuf-example
ホストをRustで書くプログラムにおけるプラグイン機能のためにwasmを使うことを考える。
wasmランタイムにwasmer, ホストとゲストのデータ受け渡しにrustのprotobuf実装prostを採用した。
neovimのfuzzy finderを念頭に大量の文字列を送り簡単な純粋な計算で文字列のスコアを返すプログラムのベンチマークをとった。

## Required
wasmerで使うのでllvm等必要かもしれない

## 構成
* src/host.rs
* guest/src/lib.r

wasmerでwasm動かしてmemory上でprotobufをやりとりする

## ベンチマーク
Ryzen 7 3750U
```
$ cargo bench
$ hyperfine 'luajit src/foo.lua'
```

N=300
```
running 3 tests
test tests::bench_native  ... bench:      17,019 ns/iter (+/- 779)
test tests::bench_native2 ... bench:      12,502 ns/iter (+/- 1,449)
test tests::bench_run     ... bench:      63,328 ns/iter (+/- 950)

test result: ok. 0 passed; 0 failed; 0 ignored; 3 measured; 0 filtered out; finished in 7.57s
```
```
Benchmark #1: luajit src/foo.lua
  Time (mean ± σ):       1.5 ms ±   0.6 ms    [User: 0.9 ms, System: 1.5 ms]
  Range (min … max):     0.0 ms …   2.9 ms    569 runs

  Warning: Command took less than 5 ms to complete. Results might be inaccurate.
```

N=300000
```
running 3 tests
test tests::bench_native  ... bench:  23,648,070 ns/iter (+/- 1,033,254)
test tests::bench_native2 ... bench:  12,313,710 ns/iter (+/- 423,131)
test tests::bench_run     ... bench:  66,752,003 ns/iter (+/- 797,943)

test result: ok. 0 passed; 0 failed; 0 ignored; 3 measured; 0 filtered out; finished in 31.17s
```
```
Benchmark #1: luajit src/foo.lua
  Time (mean ± σ):      46.1 ms ±  16.1 ms    [User: 44.9 ms, System: 1.6 ms]
  Range (min … max):    17.0 ms …  59.0 ms    48 runs
```

luaは起動時間も入ってこの時間なのでとても速い。
neovimにおいてluaインターフェイスはrpcより速いのでluaで書いたほうが速くなりそう。
