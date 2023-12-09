use halo2_proofs::circuit::Value;
use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Cell, Chip, Layouter, SimpleFloorPlanner},
    plonk::{Advice, Assigned, Circuit, Column, ConstraintSystem, Error, Fixed, Instance},
    poly::Rotation,
};
use std::marker::PhantomData;

/// Configuration of columns
/// l*sl + r*sr + (l*r)*sm - o*s0 + sc + PI = 0
struct Config {
    l: Column<Advice>,
    r: Column<Advice>,
    o: Column<Advice>,

    sl: Column<Fixed>,
    sr: Column<Fixed>,
    so: Column<Fixed>,
    sm: Column<Fixed>,
    sc: Column<Fixed>,
    PI: Column<Fixed>,
}

struct CustomChip<F: FieldExt> {
    config: Config,
    marker: PhantomData<F>,
}

impl<F: FieldExt> CustomChip<F> {
    fn new(config: Config) -> CustomChip<F> {
        CustomChip {
            config: config,
            marker: Default::default(),
        }
    }
}

trait Composer<F: FieldExt> {
    fn raw_add<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        f: F,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>;

    fn raw_multiply<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        f: F,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>;

    fn copy(&self, layouter: &mut impl Layouter<F>, a: Cell, b: Cell) -> Result<(), Error>;
    
    fn expose_public(&self, layouter: &mut impl Layouter<F>, cell: Cell, row: usize);
}

fn main() {
    println!("Hello, world!");
}
