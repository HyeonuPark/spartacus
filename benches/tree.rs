#![feature(test)]
#![allow(warnings)]

extern crate test;
extern crate rand;
#[macro_use]
extern crate spartacus;

use std::collections::BTreeMap;

use test::{Bencher, black_box};
use rand::{Rng, thread_rng};

use spartacus::arena::{Arena, BoxArena};
use spartacus::arena::vec_arena::{VecArena, Boxed as VecBoxed};

use spartacus::tree::TreeMap;
use spartacus::tree::rule::{Noop, RevTreap};

// Test macros are copied from rust-lang repository
// https://github.com/rust-lang/rust/blob/9bea79bd5ef492cf2c24e098ac93638446cb4860/src/liballoc/benches/btree/map.rs

// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

macro_rules! map_insert_rand_bench {
    ($name: ident, $n: expr, $map: ty) => (
        #[bench]
        pub fn $name(b: &mut Bencher) {
            let n: usize = $n;
            let mut map: $map = Default::default();
            // setup
            let mut rng = thread_rng();

            for _ in 0..n {
                let i = rng.gen::<usize>() % n;
                map.insert(i, i);
            }

            // measure
            b.iter(|| {
                let k = rng.gen::<usize>() % n;
                map.insert(k, k);
                map.remove(&k);
            });
            black_box(map);
        }
    )
}

macro_rules! map_insert_seq_bench {
    ($name: ident, $n: expr, $map: ty) => (
        #[bench]
        pub fn $name(b: &mut Bencher) {
            let mut map: $map = Default::default();
            let n: usize = $n;
            // setup
            for i in 0..n {
                map.insert(i * 2, i * 2);
            }

            // measure
            let mut i = 1;
            b.iter(|| {
                map.insert(i, i);
                map.remove(&i);
                i = (i + 2) % n;
            });
            black_box(map);
        }
    )
}

macro_rules! map_find_rand_bench {
    ($name: ident, $n: expr, $map: ty) => (
        #[bench]
        pub fn $name(b: &mut Bencher) {
            let mut map: $map = Default::default();
            let n: usize = $n;

            // setup
            let mut rng = thread_rng();
            let mut keys: Vec<_> = (0..n).map(|_| rng.gen::<usize>() % n).collect();

            for &k in &keys {
                map.insert(k, k);
            }

            rng.shuffle(&mut keys);

            // measure
            let mut i = 0;
            b.iter(|| {
                let t = map.get(&keys[i]);
                i = (i + 1) % n;
                black_box(t);
            })
        }
    )
}

macro_rules! map_find_seq_bench {
    ($name: ident, $n: expr, $map: ty) => (
        #[bench]
        pub fn $name(b: &mut Bencher) {
            let mut map: $map = Default::default();
            let n: usize = $n;

            // setup
            for i in 0..n {
                map.insert(i, i);
            }

            // measure
            let mut i = 0;
            b.iter(|| {
                let x = map.get(&i);
                i = (i + 1) % n;
                black_box(x);
            })
        }
    )
}

type StdBTree = BTreeMap<usize, usize>;

treemap!{BoxBst, usize, usize, Noop, BoxArena, Box, I1}
treemap!{VecBst, usize, usize, Noop, VecArena, VecBoxed, I2}
treemap!{BoxTreap, usize, usize, RevTreap, BoxArena, Box, I3}
treemap!{VecTreap, usize, usize, RevTreap, VecArena, VecBoxed, I4}

map_insert_rand_bench!{insert_rand_100_std_btree, 100, StdBTree}
map_insert_rand_bench!{insert_rand_100_box_bst,   100, BoxBst}
map_insert_rand_bench!{insert_rand_100_box_treap, 100, BoxTreap}
map_insert_rand_bench!{insert_rand_100_vec_bst,   100, VecBst}
map_insert_rand_bench!{insert_rand_100_vec_treap, 100, VecTreap}

map_insert_rand_bench!{insert_rand_10000_std_btree, 10000, StdBTree}
map_insert_rand_bench!{insert_rand_10000_box_bst,   10000, BoxBst}
map_insert_rand_bench!{insert_rand_10000_box_treap, 10000, BoxTreap}
map_insert_rand_bench!{insert_rand_10000_vec_bst,   10000, VecBst}
map_insert_rand_bench!{insert_rand_10000_vec_treap, 10000, VecTreap}

map_insert_rand_bench!{insert_rand_1000000_std_btree, 1000000, StdBTree}
map_insert_rand_bench!{insert_rand_1000000_box_treap, 1000000, BoxTreap}
map_insert_rand_bench!{insert_rand_1000000_vec_treap, 1000000, VecTreap}

map_insert_seq_bench!{insert_seq_100_std_btree, 100, StdBTree}
map_insert_seq_bench!{insert_seq_100_box_bst,   100, BoxBst}
map_insert_seq_bench!{insert_seq_100_box_treap, 100, BoxTreap}
map_insert_seq_bench!{insert_seq_100_vec_bst,   100, VecBst}
map_insert_seq_bench!{insert_seq_100_vec_treap, 100, VecTreap}

map_insert_seq_bench!{insert_seq_10000_std_btree, 10000, StdBTree}
map_insert_seq_bench!{insert_seq_10000_box_bst,   10000, BoxBst}
map_insert_seq_bench!{insert_seq_10000_box_treap, 10000, BoxTreap}
map_insert_seq_bench!{insert_seq_10000_vec_bst,   10000, VecBst}
map_insert_seq_bench!{insert_seq_10000_vec_treap, 10000, VecTreap}

map_insert_seq_bench!{insert_seq_1000000_std_btree, 1000000, StdBTree}
map_insert_seq_bench!{insert_seq_1000000_box_treap, 1000000, BoxTreap}
map_insert_seq_bench!{insert_seq_1000000_vec_treap, 1000000, VecTreap}

map_find_rand_bench!{find_rand_100_std_btree, 100, StdBTree}
map_find_rand_bench!{find_rand_100_box_bst,   100, BoxBst}
map_find_rand_bench!{find_rand_100_box_treap, 100, BoxTreap}
map_find_rand_bench!{find_rand_100_vec_bst,   100, VecBst}
map_find_rand_bench!{find_rand_100_vec_treap, 100, VecTreap}

map_find_rand_bench!{find_rand_10000_std_btree, 10000, StdBTree}
map_find_rand_bench!{find_rand_10000_box_bst,   10000, BoxBst}
map_find_rand_bench!{find_rand_10000_box_treap, 10000, BoxTreap}
map_find_rand_bench!{find_rand_10000_vec_bst,   10000, VecBst}
map_find_rand_bench!{find_rand_10000_vec_treap, 10000, VecTreap}

map_find_rand_bench!{find_rand_1000000_std_btree, 1000000, StdBTree}
map_find_rand_bench!{find_rand_1000000_box_treap, 1000000, BoxTreap}
map_find_rand_bench!{find_rand_1000000_vec_treap, 1000000, VecTreap}

map_find_seq_bench!{find_seq_100_std_btree, 100, StdBTree}
map_find_seq_bench!{find_seq_100_box_bst,   100, BoxBst}
map_find_seq_bench!{find_seq_100_box_treap, 100, BoxTreap}
map_find_seq_bench!{find_seq_100_vec_bst,   100, VecBst}
map_find_seq_bench!{find_seq_100_vec_treap, 100, VecTreap}

map_find_seq_bench!{find_seq_10000_std_btree, 10000, StdBTree}
map_find_seq_bench!{find_seq_10000_box_bst,   10000, BoxBst}
map_find_seq_bench!{find_seq_10000_box_treap, 10000, BoxTreap}
map_find_seq_bench!{find_seq_10000_vec_bst,   10000, VecBst}
map_find_seq_bench!{find_seq_10000_vec_treap, 10000, VecTreap}

map_find_seq_bench!{find_seq_1000000_std_btree, 1000000, StdBTree}
map_find_seq_bench!{find_seq_1000000_box_treap, 1000000, BoxTreap}
map_find_seq_bench!{find_seq_1000000_vec_treap, 1000000, VecTreap}
