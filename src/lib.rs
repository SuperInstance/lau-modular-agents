//! # lau-modular-agents
//!
//! Modular lattice theory for agents — the algebraic structure of capability hierarchies.
//!
//! This crate provides:
//! - **Lattice**: poset with join (∨) and meet (∧)
//! - **Modular lattice**: if a ≤ b then a ∨ (x ∧ b) = (a ∨ x) ∧ b
//! - **Distributive lattice**: a ∧ (b ∨ c) = (a ∧ b) ∨ (a ∧ c)
//! - **Boolean algebra**: complemented distributive lattice
//! - **Subgroup lattice**: lattice of subgroups of an agent group
//! - **Lattice of subspaces**: subspaces of an agent vector space form a modular lattice
//! - **Jordan-Hölder theorem**: composition series length is invariant
//! - **Krull-Schmidt theorem**: decomposition into indecomposables is unique
//! - **Modular law and Dedekind's theorem**
//! - **Application**: capability hierarchy as modular lattice

pub mod lattice;
pub mod modular;
pub mod distributive;
pub mod boolean;
pub mod subgroup;
pub mod subspace;
pub mod composition;
pub mod krull_schmidt;
pub mod dedekind;
pub mod capability;

pub use lattice::{Lattice, LatticeElement, FiniteLattice, HasseDiagram};
pub use modular::{ModularLattice, check_modular_law};
pub use distributive::{DistributiveLattice, check_distributive_law};
pub use boolean::{BooleanAlgebra, Complement};
pub use subgroup::{SubgroupLattice, Subgroup};
pub use subspace::{SubspaceLattice, Subspace};
pub use composition::{CompositionSeries, jordan_holder_length};
pub use krull_schmidt::{KrullSchmidtDecomposition, Indecomposable};
pub use dedekind::{dedekind_modular_law, verify_dedekind_theorem};
pub use capability::{CapabilityLattice, Capability, AgentCapabilityHierarchy};
