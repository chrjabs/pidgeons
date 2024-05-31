//! # Pidgeons
//!
//! A proof logging library for [VeriPB](https://gitlab.com/MIAOresearch/software/VeriPB).
//!
//! This library is a simple abstraction layer for writing proofs checkable with VeriPB.
//!
//! ## Coverage of VeriPB Syntax
//!
//! - [x] `f`
//! - [x] `pol`
//! - [x] `rup`
//! - [x] `del`
//! - [x] `delc`
//! - [x] `deld`
//! - [x] `obju`
//! - [x] `red`
//! - [x] `dom`
//! - [x] `core`
//! - [x] `output`
//! - [x] `conclusion`
//! - [ ] Subproofs
//! - [x] `e`
//! - [x] `ea`
//! - [x] `eobj`
//! - [x] `i`
//! - [x] `ia`
//! - [ ] `#`
//! - [ ] `w`
//! - [ ] `strengthening_to_core`
//! - [ ] `def_order`
//! - [ ] `load_order`
#![warn(missing_docs)]
#![warn(clippy::pedantic)]

use std::{
    io,
    ops::{Bound, RangeBounds},
};

use itertools::Itertools;

mod types;
pub use types::{
    AbsConstraintId, Axiom, Conclusion, ConstraintId, ObjectiveUpdate, OutputGuarantee, OutputType,
    ProblemType, ProofGoal, ProofGoalId, RelConstraintId, SubProof, Substitution,
};

mod ops;
pub use ops::OperationSequence;

macro_rules! unreachable_err {
    ($res:expr) => {{
        match $res {
            Ok(v) => v,
            Err(_) => unreachable!(),
        }
    }};
}
pub(crate) use unreachable_err;

/// A type representing a VeriPB proof.
///
/// This type represents the main API of this library.
#[derive(Clone, Debug)]
pub struct Proof<Writer> {
    /// Where the proof is written to
    writer: Writer,
    /// The next free constraint ID
    next_id: AbsConstraintId,
    /// The proofs problem type
    problem_type: ProblemType,
}

impl<Writer> Proof<Writer>
where
    Writer: io::Write,
{
    /// Initializes a proof with a given writer
    ///
    /// # Proof Log
    ///
    /// This writes the proof header and an `f`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn new(mut writer: Writer, num_constraints: usize, optimization: bool) -> io::Result<Self> {
        writeln!(writer, "pseudo-Boolean proof version 2.0")?;
        let mut this = Self {
            writer,
            next_id: AbsConstraintId(unreachable_err!((num_constraints + 1).try_into())),
            problem_type: ProblemType::default(),
        };
        if optimization {
            this.problem_type = ProblemType::Optimization;
        }
        this.verify_num_constraints(num_constraints)?;
        Ok(this)
    }

    /// Gets a new [`ConstraintId`] and increments the counter
    #[must_use]
    fn new_id(&mut self) -> AbsConstraintId {
        let id = self.next_id;
        self.next_id = AbsConstraintId(unreachable_err!(
            (usize::from(self.next_id.0) + 1).try_into()
        ));
        id
    }

    /// Adds a line to verify the number of constraints in the proof
    ///
    /// Note that equality constraints count as two constraints
    ///
    /// # Proof Log
    ///
    /// This writes an `f`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn verify_num_constraints(&mut self, num_constraints: usize) -> io::Result<()> {
        writeln!(self.writer, "f {num_constraints}")
    }

    /// Adds an arbitraty comment to the proof
    ///
    /// # Proof Log
    ///
    /// Adds one or more `*` lines
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn comment(&mut self, comment: &str) -> io::Result<()> {
        for line in comment.lines() {
            writeln!(self.writer, "* {line}")?;
        }
        Ok(())
    }

    /// Adds a new constraint that is derived via a sequence of operations and returns its
    /// [`AbsConstraintId`]
    ///
    /// # Proof Log
    ///
    /// Adds a `pol`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn operations(&mut self, operations: &OperationSequence) -> io::Result<AbsConstraintId> {
        writeln!(self.writer, "pol {operations}")?;
        Ok(self.new_id())
    }

    /// Adds a constraint implied by reverse unit propagation and returns its [`AbsConstraintId`]
    ///
    /// # Proof Log
    ///
    /// Adds a `rup`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    ///
    /// # Panics
    ///
    /// If `hint` is not [`None`] but empty.
    pub fn reverse_unit_prop<C: ConstraintLike>(
        &mut self,
        constr: &C,
        hint: Option<&[ConstraintId]>,
    ) -> io::Result<AbsConstraintId> {
        if let Some(hint) = hint {
            assert!(!hint.is_empty());
            writeln!(
                self.writer,
                "rup {} ; {}",
                constr.constr_str(),
                hint.iter().format(" ")
            )?;
        } else {
            writeln!(self.writer, "rup {} ;", constr.constr_str())?;
        }
        Ok(self.new_id())
    }

    /// Deletes a set of constraint by their [`ConstraintId`]s
    ///
    /// # Proof Log
    ///
    /// Adds a `del id`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    ///
    /// # Panics
    ///
    /// If `ids` is empty.
    pub fn delete_ids(&mut self, ids: &[ConstraintId]) -> io::Result<()> {
        assert!(!ids.is_empty());
        writeln!(self.writer, "del id {}", ids.iter().format(" "))
    }

    /// Deletes an explicitly specified constraint
    ///
    /// # Proof Log
    ///
    /// Adds a `del spec`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn delete_constr<C: ConstraintLike>(&mut self, constr: &C) -> io::Result<()> {
        writeln!(self.writer, "del spec {} ;", constr.constr_str())
    }

    /// Deletes a a [`ConstraintId`] range
    ///
    /// # Proof Log
    ///
    /// Adds a `del range`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    ///
    /// # Panics
    ///
    /// If `range` is empty.
    pub fn delete_id_range<R: RangeBounds<ConstraintId>>(&mut self, range: R) -> io::Result<()> {
        let range_start = match range.start_bound() {
            Bound::Included(b) => *b,
            Bound::Excluded(b) => b.increment(self.next_id),
            Bound::Unbounded => ConstraintId::Abs(AbsConstraintId::default()),
        };
        let range_end = match range.end_bound() {
            Bound::Included(b) => b.increment(self.next_id),
            Bound::Excluded(b) => *b,
            Bound::Unbounded => ConstraintId::Abs(self.next_id),
        };
        assert!(range_start.less(range_end, self.next_id));
        writeln!(self.writer, "del range {range_start} {range_end}")
    }

    /// Deletes a set of core constraint by their [`ConstraintId`]s
    ///
    /// # Proof Log
    ///
    /// Adds a `delc`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    ///
    /// # Panics
    ///
    /// If `ids` is empty.
    pub fn delete_core_ids(&mut self, ids: &[ConstraintId]) -> io::Result<()> {
        assert!(!ids.is_empty());
        writeln!(self.writer, "delc {}", ids.iter().format(" "))
    }

    /// Deletes a set of derived constraint by their [`ConstraintId`]s
    ///
    /// # Proof Log
    ///
    /// Adds a `delc`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    ///
    /// # Panics
    ///
    /// If `ids` is empty.
    pub fn delete_derived_ids(&mut self, ids: &[ConstraintId]) -> io::Result<()> {
        assert!(!ids.is_empty());
        writeln!(self.writer, "deld {}", ids.iter().format(" "))
    }

    /// Updates the objective in the proof
    ///
    /// # Proof Log
    ///
    /// Adds a `obju`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    ///
    /// # Panics
    ///
    /// If the problem is not an optimization problem.
    pub fn update_objective(&mut self, update: &ObjectiveUpdate) -> io::Result<()> {
        assert!(matches!(self.problem_type, ProblemType::Optimization));
        writeln!(self.writer, "obju {update}")
    }

    /// Adds a set of substitutions
    ///
    /// # Proof Log
    ///
    /// Adds a substitution line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails
    ///
    /// # Panics
    ///
    /// If `subs` is empty.
    pub fn substitute(&mut self, subs: &[Substitution]) -> io::Result<()> {
        assert!(!subs.is_empty());
        writeln!(self.writer, "{}", subs.iter().format(" "))
    }

    /// Adds a constraint that is redundant, checked via redundance based strengthening
    ///
    /// # Proof Log
    ///
    /// Adds a `red`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn redundant<C: ConstraintLike>(
        &mut self,
        constr: &C,
        subs: &[Substitution],
    ) -> io::Result<AbsConstraintId> {
        writeln!(
            self.writer,
            "red {} ; {}",
            constr.constr_str(),
            subs.iter().format(" ")
        )?;
        Ok(self.new_id())
    }

    /// Adds a constraint that is redundant, checked via dominance
    ///
    /// # Proof Log
    ///
    /// Adds a `dom`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn dominated<C: ConstraintLike>(
        &mut self,
        constr: &C,
        subs: &[Substitution],
    ) -> io::Result<AbsConstraintId> {
        writeln!(
            self.writer,
            "dom {} ; {}",
            constr.constr_str(),
            subs.iter().format(" ")
        )?;
        Ok(self.new_id())
    }

    /// Moves constraints to the core set by [`ConstraintId`]
    ///
    /// # Proof Log
    ///
    /// Adds a `core id` line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    ///
    /// # Panics
    ///
    /// If `ids` is empty.
    pub fn move_id_to_core(&mut self, ids: &[ConstraintId]) -> io::Result<()> {
        assert!(!ids.is_empty());
        writeln!(self.writer, "core id {}", ids.iter().format(" "))
    }

    /// Moves a range of constraints to the core set
    ///
    /// # Proof Log
    ///
    /// Adds a `core id` line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    ///
    /// # Panics
    ///
    /// If `range` is empty.
    pub fn move_range_to_core<R: RangeBounds<ConstraintId>>(&mut self, range: R) -> io::Result<()> {
        let range_start = match range.start_bound() {
            Bound::Included(b) => *b,
            Bound::Excluded(b) => b.increment(self.next_id),
            Bound::Unbounded => ConstraintId::Abs(AbsConstraintId::default()),
        };
        let range_end = match range.end_bound() {
            Bound::Included(b) => b.increment(self.next_id),
            Bound::Excluded(b) => *b,
            Bound::Unbounded => ConstraintId::Abs(self.next_id),
        };
        assert!(range_start.less(range_end, self.next_id));
        writeln!(self.writer, "core range {range_start} {range_end}")
    }

    /// Adds an output section to the proof
    ///
    /// # Proof Log
    ///
    /// Writes an `output` line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn output(&mut self, guarantee: OutputGuarantee, r#type: OutputType) -> io::Result<()> {
        writeln!(self.writer, "output {guarantee} {type}")
    }

    /// Adds a conclusion section to the proof
    ///
    /// # Proof Log
    ///
    /// Writes a `conclusion` line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn conclusion(&mut self, conclusion: &Conclusion) -> io::Result<()> {
        writeln!(self.writer, "conclusion {conclusion}")
    }

    /// Ends the proof
    ///
    /// # Proof Log
    ///
    /// Writes an `end` line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn end(mut self) -> io::Result<()> {
        writeln!(self.writer, "end pseudo-Boolean proof")
    }

    /// Concludes the proof by adding the output and conclusions sections and ending the proof.
    ///
    /// # Proof Log
    ///
    /// Writes `output`, `conclusion`, and `end` lines.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn conclude(
        mut self,
        guarantee: OutputGuarantee,
        r#type: OutputType,
        conclusion: &Conclusion,
    ) -> io::Result<()> {
        self.output(guarantee, r#type)?;
        self.conclusion(conclusion)?;
        self.end()
    }

    /// Checks whether a constraint is equal to a constraint that is already in the database
    ///
    /// # Proof Log
    ///
    /// Writes a `e`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn equals<C: ConstraintLike>(
        &mut self,
        constraint: &C,
        equals: Option<ConstraintId>,
    ) -> io::Result<()> {
        if let Some(id) = equals {
            writeln!(self.writer, "e {} ; {id}", constraint.constr_str())
        } else {
            writeln!(self.writer, "e {} ;", constraint.constr_str())
        }
    }

    /// Checks whether a constraint is equal to a constraint that is already in the database and
    /// adds the constraint
    ///
    /// # Proof Log
    ///
    /// Writes a `ea`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn equals_add<C: ConstraintLike>(
        &mut self,
        constraint: &C,
        equals: Option<ConstraintId>,
    ) -> io::Result<AbsConstraintId> {
        if let Some(id) = equals {
            writeln!(self.writer, "ea {} ; {id}", constraint.constr_str())?;
        } else {
            writeln!(self.writer, "ea {} ;", constraint.constr_str())?;
        }
        Ok(self.new_id())
    }

    /// Checks whether the given objective is equal to the current objective
    ///
    /// # Proof Log
    ///
    /// Writes a `eobj`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    ///
    /// # Panics
    ///
    /// If the problem is not an optimization problem.
    pub fn obj_equals<O: ObjectiveLike>(&mut self, objective: &O) -> io::Result<()> {
        assert!(matches!(self.problem_type, ProblemType::Optimization));
        writeln!(self.writer, "eobj {} ;", objective.obj_str())
    }

    /// Checks whether the given constraint is implied
    ///
    /// # Proof Log
    ///
    /// Writes an `i`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn implied<C: ConstraintLike>(
        &mut self,
        constraint: &C,
        implicant: Option<ConstraintId>,
    ) -> io::Result<()> {
        if let Some(id) = implicant {
            writeln!(self.writer, "i {} ; {id}", constraint.constr_str())
        } else {
            writeln!(self.writer, "i {} ;", constraint.constr_str())
        }
    }

    /// Checks whether the given constraint is implied and adds it
    ///
    /// # Proof Log
    ///
    /// Writes an `is`-rule line.
    ///
    /// # Errors
    ///
    /// If writing the proof fails.
    pub fn implied_add<C: ConstraintLike>(
        &mut self,
        constraint: &C,
        implicant: Option<ConstraintId>,
    ) -> io::Result<AbsConstraintId> {
        if let Some(id) = implicant {
            writeln!(self.writer, "ia {} ; {id}", constraint.constr_str())?;
        } else {
            writeln!(self.writer, "ia {} ;", constraint.constr_str())?;
        }
        Ok(self.new_id())
    }
}

/// Trait that needs to be implemented for types used as variables
pub trait VarLike {
    /// Gets a string representation of the variable
    ///
    /// # Contract
    ///
    /// A valid VeriPB variable identifier must be returned
    fn var_str(&self) -> String;

    /// Gets a positive axiom of the variable for an operation sequence
    fn pos_axiom(&self) -> Axiom {
        Axiom {
            neg: false,
            var: self.var_str(),
        }
    }

    /// Gets a negative axiom of the variable for an operation sequence
    fn neg_axiom(&self) -> Axiom {
        Axiom {
            neg: true,
            var: self.var_str(),
        }
    }

    /// Substitutes the variables with a fixed value
    fn substitute_fixed(&self, value: bool) -> Substitution {
        Substitution {
            var: self.var_str(),
            sub: value.into(),
        }
    }

    /// Substitutes the variable with a literal
    fn substitute_literal(&self, literal: Axiom) -> Substitution {
        Substitution {
            var: self.var_str(),
            sub: types::SubstituteWith::Lit(literal),
        }
    }
}

impl VarLike for String {
    fn var_str(&self) -> String {
        self.clone()
    }
}

impl VarLike for str {
    fn var_str(&self) -> String {
        String::from(self)
    }
}

/// Trait that needs to be implemented for types used as constraints
pub trait ConstraintLike {
    /// Gets a string representation of the constraint
    ///
    /// # Contract
    ///
    /// Must return a valid OPB-style constraint.
    fn constr_str(&self) -> String;
}

/// Trait that needs to be implemented for types used as objectives
pub trait ObjectiveLike {
    /// Gets a string representation of the objective
    ///
    /// # Contract
    ///
    /// Must return a valid OPB-style objetive.
    fn obj_str(&self) -> String;
}

impl<V, Iter> ObjectiveLike for Iter
where
    V: VarLike,
    Iter: IntoIterator<Item = (isize, V)> + Clone,
{
    fn obj_str(&self) -> String {
        format!(
            "{}",
            self.clone()
                .into_iter()
                .map(|(cf, v)| format!("{cf} {}", v.var_str()))
                .format(" ")
        )
    }
}
