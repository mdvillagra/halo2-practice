use halo2_proofs::circuit::Value;
use halo2_proofs::dev::MockProver;
use halo2_proofs::halo2curves::bn256::Fr as Fp;
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
#[derive(Debug, Clone)]
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

impl<F: FieldExt> Circuit<F> for SampleCircuit<F> {
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

            let sl = meta.query_fixed(sl, Rotation::cur());
            let sr = meta.query_fixed(sr, Rotation::cur());
            let so = meta.query_fixed(so, Rotation::cur());
            let sm = meta.query_fixed(sm, Rotation::cur());
            let sc = meta.query_fixed(sc, Rotation::cur());

            vec![l.clone() * sl + r.clone() * sr + l * r * sm + (o * so * (-F::one())) + sc]
        });

        meta.create_gate("public input", |meta| {
            let l = meta.query_advice(l, Rotation::cur());
            #[allow(non_snake_case)]
            let PI = meta.query_instance(PI, Rotation::cur());
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

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let cs = CustomChip::new(config);

        let x: Value<Assigned<_>> = self.x.into();
        let y: Value<Assigned<_>> = self.y.into();
        let consty = Assigned::from(self.constant);

        let (a0, b0, c0) = cs.raw_multiply(&mut layouter, || x.map(|x| (x, x, x * x)))?;
        cs.copy(&mut layouter, a0, b0)?;

        let (a1, b1, c1) = cs.raw_multiply(&mut layouter, || y.map(|y| (y, y, y * y)))?;
        cs.copy(&mut layouter, a1, b1)?;

        let (a2, b2, c2) = cs.raw_multiply(&mut layouter, || {
            x.zip(y).map(|(x, y)| (x * x, y * y, x * x * y * y))
        })?;
        cs.copy(&mut layouter, a2, c0)?;
        cs.copy(&mut layouter, b2, c1)?;

        let (a3, b3, c3) = cs.raw_add(&mut layouter, || {
            x.zip(y)
                .map(|(x, y)| (x * x * y * y, consty, x * x * y * y + consty))
        })?;
        cs.copy(&mut layouter, a3, c2)?;

        cs.expose_public(&mut layouter, b3, 0)?;

        layouter.constrain_instance(c3, cs.config.PI, 1)?;

        Ok(())
    }
}

fn main() {
    let k = 4;

    let constant = Fp::from(7);
    let x = Fp::from(5);
    let y = Fp::from(9);
    let z = Fp::from(25 * 81 + 7);

    let circuit = SampleCircuit {
        x: Value::known(x),
        y: Value::known(y),
        constant: constant,
    };
    let mut public_inputs = vec![constant, z];

    let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));
}
