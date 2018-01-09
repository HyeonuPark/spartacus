use std::mem::replace;

use tree::Indirect;

pub trait Rule: Default {
    fn update<K, V, I>(node: &mut I) where
        K: Ord,
        I: Indirect<K, V, Self>;
}

#[derive(Default)]
pub struct Noop;

impl Rule for Noop {
    fn update<K, V, I>(_node: &mut I) where
        K: Ord,
        I: Indirect<K, V, Self>
    {}
}

#[derive(Debug)]
pub struct RotateEmptyLeg;

pub trait Rotate {
    fn rotate_left<K, V, R>(&mut self) -> Result<(), RotateEmptyLeg> where
        K: Ord, R: Rule, Self: Indirect<K, V, R>;
    fn rotate_right<K, V, R>(&mut self) -> Result<(), RotateEmptyLeg> where
        K: Ord, R: Rule, Self: Indirect<K, V, R>;
}

impl<T> Rotate for T {
    fn rotate_left<K, V, R>(&mut self) -> Result<(), RotateEmptyLeg> where
        K: Ord, R: Rule, Self: Indirect<K, V, R>
    {
        //
        //     R            B
        //    / \          / \
        //   A   B   =>   R   D
        //      / \      / \
        //     C   D    A   C
        //

        let node_b = match self.right.take() {
            None => Err(RotateEmptyLeg)?,
            Some(node) => node,
        };

        let mut node_r = replace(self, node_b);

        let edge_c = self.left.take();

        node_r.right = edge_c;

        self.left = Some(node_r);

        Ok(())
    }

    fn rotate_right<K, V, R>(&mut self) -> Result<(), RotateEmptyLeg> where
        K: Ord, R: Rule, Self: Indirect<K, V, R>
    {
        //
        //     R            A
        //    / \          / \
        //   A   B   =>   C   R
        //  / \              / \
        // C   D            D   B
        //

        let node_a = match self.left.take() {
            None => Err(RotateEmptyLeg)?,
            Some(node) => node,
        };

        let mut node_r = replace(self, node_a);

        let edge_d = self.right.take();

        node_r.left = edge_d;

        self.right = Some(node_r);

        Ok(())
    }
}
