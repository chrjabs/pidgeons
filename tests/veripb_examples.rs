//! # Integration Tests Copied from VeriPB

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    process::Command,
};

use pidgeons::{Conclusion, ConstraintId as Id, OutputGuarantee, Proof, VarLike};

fn print_file<P: AsRef<Path>>(path: P) {
    for line in BufReader::new(File::open(path).expect("could not open file")).lines() {
        println!("{}", line.unwrap());
    }
}

fn verify_proof<P1: AsRef<Path>, P2: AsRef<Path>>(instance: P1, proof: P2) {
    let out = Command::new("veripb")
        .arg(instance.as_ref())
        .arg(proof.as_ref())
        .output()
        .expect("failed to run veripb");
    if out.status.success() {
        return;
    }
    print_file(proof);
    panic!("verification failed: {out:?}")
}

fn new_proof(num_constraints: usize, optimization: bool) -> Proof<tempfile::NamedTempFile> {
    let file = tempfile::NamedTempFile::new().expect("failed to create temporary proof file");
    pidgeons::Proof::new(file, num_constraints, optimization).expect("failed to start proof")
}

#[test]
fn all_diff() {
    let mut proof = new_proof(15, false);
    let new1 = proof
        .operations(&(Id::abs(3) + Id::abs(4) + Id::abs(5)))
        .unwrap();
    let new2 = proof
        .operations(
            &(Id::abs(14)
                + Id::abs(15)
                + "y_x1_8".pos_axiom()
                + "y_x2_8".pos_axiom()
                + "y_x1_9".pos_axiom()
                + "y_x2_9".pos_axiom()),
        )
        .unwrap();
    let contrad = proof
        .operations(&(Id::from(new1) + Id::from(new2)))
        .unwrap();
    let proof_file = proof
        .conclude(
            OutputGuarantee::None,
            &Conclusion::Unsat(Some(contrad.into())),
        )
        .unwrap();
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    verify_proof(format!("{manifest}/data/all_diff.opb"), proof_file.path());
}

#[test]
fn implication_weaker() {
    let mut proof = new_proof(1, false);
    proof
        .implied_add(&"1 x1 2 x2 4 x3 >= 3", Some(Id::abs(1)))
        .unwrap();
    proof
        .equals(&"1 x1 2 x2 4 x3 >= 3", Some(Id::last(1)))
        .unwrap();
    let proof_file = proof
        .conclude(OutputGuarantee::None, &Conclusion::None)
        .unwrap();
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    verify_proof(
        format!("{manifest}/data/implication_weaker.opb"),
        proof_file.path(),
    );
}
