use halo2_proofs::circuit::Value;
use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Cell, Chip, Layouter, SimpleFloorPlanner},
    plonk::{Advice, Assigned, Circuit, Column, ConstraintSystem, Error, Fixed, Instance},
    poly::Rotation,
};
use std::default;
use std::marker::PhantomData;

/// Configuration of columns
/// l*sl + r*sr + (l*r)*sm - o*so + sc + PI = 0
struct CustomConfig {
    l: Column<Advice>,
    r: Column<Advice>,
    o: Column<Advice>,

    sl: Column<Fixed>,
    sr: Column<Fixed>,
    so: Column<Fixed>,
    sm: Column<Fixed>,
    sc: Column<Fixed>,

    PI: Column<Instance>,
}

struct CustomChip<F: FieldExt> {
    config: CustomConfig,
    marker: PhantomData<F>,
}

impl<F: FieldExt> CustomChip<F> {
    fn new(config: CustomConfig) -> CustomChip<F> {
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
        f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>;

    fn raw_multiply<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>;

    fn copy(&self, layouter: &mut impl Layouter<F>, a: Cell, b: Cell) -> Result<(), Error>;

    fn expose_public(
        &self,
        layouter: &mut impl Layouter<F>,
        cell: Cell,
        row: usize,
    ) -> Result<(), Error>;
}

impl<F: FieldExt> Composer<F> for CustomChip<F> {
    fn raw_multiply<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        mut f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>,
    {
        layouter.assign_region(
            || "mul",
            |mut region| {
                let mut values = None;
                let lhs = region.assign_advice(
                    || "lhs",
                    self.config.l,
                    0,
                    || {
                        values = Some(f());
                        values.unwrap().map(|v| v.0)
                    },
                )?;
                let rhs = region.assign_advice(
                    || "rhs",
                    self.config.r,
                    0,
                    || values.unwrap().map(|v| v.1),
                )?;

                let out = region.assign_advice(
                    || "out",
                    self.config.o,
                    0,
                    || values.unwrap().map(|v| v.2),
                )?;

                region.assign_fixed(|| "m", self.config.sm, 0, || Value::known(F::one()))?;
                region.assign_fixed(|| "o", self.config.so, 0, || Value::known(F::one()))?;

                Ok((lhs.cell(), rhs.cell(), out.cell()))
            },
        )
    }

    fn raw_add<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        mut f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>,
    {
        layouter.assign_region(
            || "add",
            |mut region| {
                let mut values = None;
                let lhs = region.assign_advice(
                    || "lhs",
                    self.config.l,
                    0,
                    || {
                        values = Some(f());
                        values.unwrap().map(|v| v.0)
                    },
                )?;
                let rhs = region.assign_advice(
                    || "rhs",
                    self.config.l,
                    0,
                    || values.unwrap().map(|v| v.0),
                )?;
                let out = region.assign_advice(
                    || "out",
                    self.config.l,
                    0,
                    || values.unwrap().map(|v| v.0),
                )?;

                region.assign_fixed(|| "l", self.config.sl, 0, || Value::known(F::one()))?;
                region.assign_fixed(|| "r", self.config.sr, 0, || Value::known(F::one()))?;
                region.assign_fixed(|| "o", self.config.so, 0, || Value::known(F::one()))?;

                Ok((lhs.cell(), rhs.cell(), out.cell()))
            },
        )
    }

    fn copy(&self, layouter: &mut impl Layouter<F>, a: Cell, b: Cell) -> Result<(), Error> {
        layouter.assign_region(
            || "copy",
            |mut region| {
                region.constrain_equal(a, b)?;
                region.constrain_equal(a, b)
            },
        )
    }

    fn expose_public(
        &self,
        layouter: &mut impl Layouter<F>,
        cell: Cell,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell, self.config.PI, row)
    }
}

#[derive(Default)]
struct SampleCircuit<F: FieldExt> {
    x: Value<F>,
    y: Value<F>,
    constant: F,
}

impl<F: FieldExt> Circuit for SampleCircuit<F> {
    type Config = CustomConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let l = meta.advice_column();
        let r = meta.advice_column();
        let o = meta.advice_column();

        meta.enable_equality(l);
        meta.enable_equality(r);
        meta.enable_equality(o);

        let sm = meta.fixed_column();
        let sl = meta.fixed_column();
        let sr = meta.fixed_column();
        let so = meta.fixed_column();
        let sc = meta.fixed_column();
        let sp = meta.fixed_column();

        #[allow(non_snake_case)]
        let PI = meta.instance_column();
        meta.enable_equality(PI);

        meta.create_gate("mini plonk", |meta| {
            let l = meta.query_advice(l, Rotation::cur());
            let r = meta.query_advice(r, Rotation::cur());
            let o = meta.query_advice(o, Rotation::cur());

            let sl = meta.query_advice(sl, Rotation::cur());
            let sr = meta.query_advice(sr, Rotation::cur());
            let so = meta.query_advice(so, Rotation::cur());
            let sm = meta.query_advice(sm, Rotation::cur());
            let sc = meta.query_advice(sc, Rotation::cur());

            vec![l.clone() * sl + r.clone() * sr + l * r * sm + (o * so * (-F::one())) + sc]
        });

        meta.create_gate("public input", |meta| {
            let l = meta.query_fixed(l, Rotation::cur());
            #[allow(non_snake_case)]
            let PI = meta.query_fixed(PI, Rotation::cur());
            let sp = meta.query_fixed(sp, Rotation::cur());

            vec![sp * (l - PI)]
        });

        CustomConfig {
            l,
            r,
            o,
            sl,
            sr,
            so,
            sm,
            sc,
            PI,
        }
    }

    fn synthesize(&self, config: Self::Config, layouter: impl Layouter<F>) -> Result<(), Error> {
        let cs = CustomChip::new(config);

        let x = self.x.into();
        let y = self.y.into();
        let consty = Assigned::from(self.constant);

        let (a0, b0, c0) = cs.raw_multiply(&mut layouter, || x.map(|x| (x, x, x * x)))?;
        cs.copy(&mut layouter, a0, b0)
    }
}

fn main() {
    println!("Hello, world!");
}
