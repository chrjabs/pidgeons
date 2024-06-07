<!-- cargo-rdme start -->

# Pidgeons

A proof logging library for [VeriPB](https://gitlab.com/MIAOresearch/software/VeriPB).

This library is a simple abstraction layer for writing proofs checkable with VeriPB.

## Coverage of VeriPB Syntax

- [x] `f`: [`Proof::new`]
- [x] `pol`: [`Proof::operations`]
- [x] `rup`: [`Proof::reverse_unit_prop`]
- [x] `del`: [`Proof::delete_ids`], [`Proof::delete_id_range`], [`Proof::delete_constr`]
- [x] `delc`: [`Proof::delete_core_ids`]
- [x] `deld`: [`Proof::delete_derived_ids`]
- [x] `obju`: [`Proof::update_objective`]
- [x] `red`: [`Proof::redundant`]
- [x] `dom`: [`Proof::dominated`]
- [x] `core`: [`Proof::move_ids_to_core`], [`Proof::move_range_to_core`]
- [x] `output`: [`Proof::output`], [`Proof::conclude`]
- [x] `conclusion`: [`Proof::conclusion`], [`Proof::conclude`]
- [ ] Subproofs
- [x] `e`: [`Proof::equals`]
- [x] `ea`: [`Proof::equals_add`]
- [x] `eobj`: [`Proof::obj_equals`]
- [x] `i`: [`Proof::implied`]
- [x] `ia`: [`Proof::implied_add`]
- [ ] `#`
- [ ] `w`
- [ ] `strengthening_to_core`
- [ ] `def_order`
- [ ] `load_order`

<!-- cargo-rdme end -->
