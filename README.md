# lau-modular-agents

> **Modular lattice theory for agents** — the algebraic structure of capability hierarchies, from posets to Krull-Schmidt decomposition.

## What This Does

This crate implements the theory of **modular lattices** from the ground up: general lattices, the modular law, distributive lattices, Boolean algebras, subgroup lattices, subspace lattices, the Jordan-Hölder theorem, the Krull-Schmidt theorem, and Dedekind's characterization. It then applies this theory to model **agent capability hierarchies** as modular lattices, where join = combined capability, meet = shared capability, and the modular law guarantees clean composition.

Part of the **PLATO/LAU ecosystem** — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## Key Idea

Agent capabilities form a natural hierarchy: `base ⊆ move ⊆ navigate ⊆ full`. This is a **lattice** — a partially ordered set where any two elements have a least upper bound (join, ∨) and a greatest lower bound (meet, ∧).

But not just any lattice. When capabilities compose cleanly — combining `move` and `sense` gives `navigate`, and the modular law holds — you get a **modular lattice**. Modular lattices have deep structure theorems:

- **Jordan-Hölder**: Every maximal chain has the same length (composition series are unique up to permutation)
- **Krull-Schmidt**: Every element decomposes uniquely into indecomposables
- **Dedekind**: A lattice is modular if and only if it doesn't contain the pentagon N₅

These aren't just abstract results. They tell you that your capability hierarchy has a well-defined "dimension" and a unique factorization — just like the integers have unique prime factorization.

## Install

```bash
cargo add lau-modular-agents
```

Dependencies: `nalgebra` (linear algebra for subspace lattices), `serde`/`serde_json` (serialization).

## Quick Start

```rust
use lau_modular_agents::*;
use lau_modular_agents::lattice::{FiniteLattice, LatticeElement, chain_lattice, diamond_m3_lattice, powerset_lattice};

// A chain lattice: 0 < 1 < 2 < 3
let chain = chain_lattice(4);
assert_eq!(chain.join("c0", "c2"), Some("c2"));  // max
assert_eq!(chain.meet("c0", "c2"), Some("c0"));  // min
assert!(chain.verify_axioms().is_ok());

// The diamond M₃ (modular but not distributive)
let m3 = diamond_m3_lattice();
assert!(check_modular_law(&m3).is_ok());

// The pentagon N₅ (NOT modular — Dedekind's counterexample)
let n5 = lau_modular_agents::lattice::pentagon_n5_lattice();
assert!(check_modular_law(&n5).is_err());

// Powerset lattice (distributive, hence modular)
let p = powerset_lattice(3);
assert!(check_distributive_law(&p).is_ok());

// Jordan-Hölder: all maximal chains have the same length
let result = verify_jordan_holder(&m3);
assert_eq!(result, Ok(2));

// Capability hierarchy
let robot = CapabilityLattice::build("robot-1")
    .capability("base", 0)
    .capability("move", 1)
    .capability("sense", 1)
    .capability("navigate", 2)
    .capability("full", 3)
    .base("base")
    .full("full")
    .join("move", "sense", "navigate")
    .meet("move", "sense", "base")
    // ... (set all join/meet pairs)
    .finish();

assert!(robot.hierarchy().verify_modular().is_ok());
```

## API Reference

### Lattice Core (`lattice`)

```rust
pub struct LatticeElement {
    pub label: String,
    pub rank: Option<usize>,
}

// A finite lattice with explicit join/meet tables and cover relations
pub struct FiniteLattice { /* ... */ }

impl FiniteLattice {
    pub fn new() -> Self;
    pub fn add_element(&mut self, elem: LatticeElement);
    pub fn add_cover(&mut self, a: impl Into<String>, b: impl Into<String>);
    pub fn set_join(&mut self, a: &str, b: &str, result: impl Into<String>);
    pub fn set_meet(&mut self, a: &str, b: &str, result: impl Into<String>);
    pub fn set_top(&mut self, label: impl Into<String>);
    pub fn set_bottom(&mut self, label: impl Into<String>);

    pub fn join(&self, a: &str, b: &str) -> Option<&str>;    // a ∨ b
    pub fn meet(&self, a: &str, b: &str) -> Option<&str>;    // a ∧ b
    pub fn leq(&self, a: &str, b: &str) -> bool;              // a ≤ b (transitive closure)
    pub fn lt(&self, a: &str, b: &str) -> bool;               // a < b (strict)
    pub fn covers_of(&self, a: &str) -> HashSet<&str>;
    pub fn verify_axioms(&self) -> Result<(), Vec<String>>;   // idempotent, commutative, associative, absorption
    pub fn size(&self) -> usize;
    pub fn top(&self) -> Option<&str>;
    pub fn bottom(&self) -> Option<&str>;
}

// Hasse diagram extraction
pub struct HasseDiagram {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
}
impl HasseDiagram {
    pub fn from_lattice(lattice: &FiniteLattice) -> Self;
    pub fn height(&self) -> usize;
}

// Pre-built lattices
pub fn chain_lattice(n: usize) -> FiniteLattice;
pub fn diamond_m3_lattice() -> FiniteLattice;
pub fn pentagon_n5_lattice() -> FiniteLattice;
pub fn powerset_lattice(n: usize) -> FiniteLattice;
```

### Modular Lattices (`modular`)

```rust
// Check the modular law for all triples: if a ≤ b then a ∨ (x ∧ b) = (a ∨ x) ∧ b
pub fn check_modular_law(lattice: &FiniteLattice) -> Result<(), Vec<String>>;

// A verified modular lattice (construction fails if the modular law is violated)
pub struct ModularLattice { /* ... */ }

impl ModularLattice {
    pub fn new(lattice: FiniteLattice) -> Result<Self, Vec<String>>;
    pub fn new_unchecked(lattice: FiniteLattice) -> Self;
    pub fn inner(&self) -> &FiniteLattice;
    pub fn diamond_isomorphism(&self, a: &str, b: &str, x: &str) -> Vec<(String, String)>;
}
```

### Distributive Lattices (`distributive`)

```rust
pub fn check_distributive_law(lattice: &FiniteLattice) -> Result<(), Vec<String>>;

pub struct DistributiveLattice { /* ... */ }
impl DistributiveLattice {
    pub fn new(lattice: FiniteLattice) -> Result<Self, Vec<String>>;
}
```

### Boolean Algebras (`boolean`)

```rust
pub struct Complement {
    pub element: String,
    pub complement: String,
}

pub struct BooleanAlgebra {
    pub lattice: FiniteLattice,
    pub complements: Vec<Complement>,
}

impl BooleanAlgebra {
    pub fn from_powerset(n: usize) -> Self;
    pub fn complement(&self, element: &str) -> Option<&str>;
    pub fn verify_complement_laws(&self) -> Result<(), Vec<String>>;
}
```

### Subgroup Lattice (`subgroup`)

```rust
pub struct Subgroup {
    pub elements: Vec<usize>,
    pub label: String,
}

pub struct SubgroupLattice {
    pub group_order: usize,
    pub lattice: FiniteLattice,
    pub subgroups: Vec<Subgroup>,
}

impl SubgroupLattice {
    pub fn cyclic_group(n: usize) -> Self;   // subgroups of Z/nZ
    pub fn symmetric_group_3() -> Self;       // subgroups of S₃
}
```

### Subspace Lattice (`subspace`)

```rust
pub struct Subspace {
    pub basis: Vec<Vec<f64>>,
    pub label: String,
}

pub struct SubspaceLattice {
    pub dimension: usize,
    pub field_size: usize,
    pub lattice: FiniteLattice,
    pub subspaces: Vec<Subspace>,
}

impl SubspaceLattice {
    pub fn from_subspaces(dimension: usize, subspaces: Vec<Subspace>) -> Self;
}
```

### Jordan-Hölder Theorem (`composition`)

```rust
pub struct CompositionSeries {
    pub chain: Vec<String>,
    pub length: usize,
    pub factors: Vec<(String, String)>,
}

pub fn find_maximal_chains(lattice: &FiniteLattice) -> Vec<Vec<String>>;
pub fn jordan_holder_length(lattice: &FiniteLattice) -> Option<usize>;
pub fn verify_jordan_holder(lattice: &FiniteLattice) -> Result<usize, Vec<usize>>;
pub fn all_composition_series(lattice: &FiniteLattice) -> Vec<CompositionSeries>;
```

### Krull-Schmidt Theorem (`krull_schmidt`)

```rust
pub struct KrullSchmidtDecomposition {
    pub element: String,
    pub factors: Vec<String>,
    pub is_unique: bool,
}

impl KrullSchmidtDecomposition {
    pub fn decompose(lattice: &FiniteLattice, element: &str) -> Self;
    pub fn verify_uniqueness(lattice: &FiniteLattice) -> bool;
}
```

### Dedekind's Theorem (`dedekind`)

```rust
pub fn dedekind_modular_law(lattice: &FiniteLattice, a: &str, x: &str, b: &str) -> Result<bool, String>;
pub fn contains_pentagon_n5(lattice: &FiniteLattice) -> Option<Vec<String>>;
pub fn verify_dedekind_theorem(lattice: &FiniteLattice) -> DedekindResult;

pub struct DedekindResult {
    pub is_modular: bool,
    pub contains_pentagon: bool,
    pub pentagon_witness: Option<Vec<String>>,
    pub modular_violations: Vec<String>,
    pub theorem_holds: bool,
}
```

### Capability Hierarchy (`capability`)

```rust
pub struct Capability {
    pub name: String,
    pub level: usize,
    pub description: String,
}

pub struct AgentCapabilityHierarchy {
    pub agent_id: String,
    pub capabilities: HashMap<String, Capability>,
    /* ... */
}

impl AgentCapabilityHierarchy {
    pub fn new(agent_id: impl Into<String>) -> Self;
    pub fn add_capability(&mut self, cap: Capability);
    pub fn set_subcapability(&mut self, a: &str, b: &str);
    pub fn verify_modular(&self) -> Result<(), Vec<String>>;
    pub fn composition_guarantee(&self, a: &str, x: &str, b: &str) -> bool;
    pub fn subsumes(&self, a: &str, b: &str) -> bool;
}

// Builder pattern
pub struct CapabilityLattice { /* ... */ }
impl CapabilityLattice {
    pub fn build(agent_id: impl Into<String>) -> CapabilityLatticeBuilder;
}
impl CapabilityLatticeBuilder {
    pub fn capability(self, name: &str, level: usize) -> Self;
    pub fn subcapability(self, a: &str, b: &str) -> Self;
    pub fn join(self, a: &str, b: &str, result: &str) -> Self;
    pub fn meet(self, a: &str, b: &str, result: &str) -> Self;
    pub fn base(self, name: &str) -> Self;
    pub fn full(self, name: &str) -> Self;
    pub fn finish(self) -> CapabilityLattice;
}
```

## How It Works

### Lattice Representation

Lattices are stored as explicit tables: a set of labeled elements, a `join_table` and `meet_table` mapping pairs to results, and a `covers` map encoding the Hasse diagram (cover relations). The partial order is computed by transitive closure over covers using BFS.

### Axiom Verification

`verify_axioms()` checks all four lattice axioms for all element pairs:
1. **Idempotent**: a ∨ a = a, a ∧ a = a
2. **Commutative**: a ∨ b = b ∨ a, a ∧ b = b ∧ a
3. **Associative**: (a ∨ b) ∨ c = a ∨ (b ∨ c), same for ∧
4. **Absorption**: a ∨ (a ∧ b) = a, a ∧ (a ∨ b) = a

### Modular Law Checking

For every triple (a, x, b) where a ≤ b, we verify:

```
a ∨ (x ∧ b) = (a ∨ x) ∧ b
```

If this fails for any triple, the lattice is not modular and the violations are reported with specific element labels.

### Pentagon Detection (Dedekind)

The crate searches for the pentagon N₅ embedded as a sublattice: five elements {bot, a, b, c, top} with bot < a < c < top, bot < b < top, and b incomparable to both a and c. The key signature is the modular law violation at (a, b, c).

### Jordan-Hölder Verification

All maximal chains from bottom to top are enumerated via recursive DFS over cover relations. For a modular lattice, all such chains have the same length — this is verified by `verify_jordan_holder()`.

## The Math

### Lattices

A **lattice** is a partially ordered set (poset) in which every pair of elements has a least upper bound (**join**, ∨) and a greatest lower bound (**meet**, ∧). Key axioms:
- **Idempotent**: a ∨ a = a
- **Commutative**: a ∨ b = b ∨ a
- **Associative**: (a ∨ b) ∨ c = a ∨ (b ∨ c)
- **Absorption**: a ∨ (a ∧ b) = a

The **Hasse diagram** is the directed graph of cover relations (a ≺ b means a < b with no element strictly between).

### Modular Lattices

A lattice is **modular** if for all a ≤ b and all x:

```
a ∨ (x ∧ b) = (a ∨ x) ∧ b
```

This is a weakened form of distributivity. Examples:
- Any **chain** (totally ordered set) is modular
- The **diamond M₃** is modular but not distributive
- The **pentagon N₅** is the smallest non-modular lattice

**Dedekind's theorem**: A lattice is modular ⟺ it does not contain N₅ as a sublattice.

### Distributive Lattices

A lattice is **distributive** if for all a, b, c:

```
a ∧ (b ∨ c) = (a ∧ b) ∨ (a ∧ c)
```

Every distributive lattice is modular. The powerset lattice P(S) (ordered by ⊆, with ∨ = ∪ and ∧ = ∩) is distributive.

### Boolean Algebras

A **Boolean algebra** is a complemented distributive lattice. Every element a has a complement a' satisfying a ∨ a' = ⊤ and a ∧ a' = ⊥. The powerset lattice is the canonical example: the complement of S ⊆ X is X \ S.

### Subgroup and Subspace Lattices

The lattice of **subgroups** of a group G (ordered by ⊆, with ∨ = ⟨H₁, H₂⟩ and ∧ = H₁ ∩ H₂) is always modular (by Dedekind's modular law for groups). The lattice of **subspaces** of a vector space V is also modular (but not distributive in general).

### Jordan-Hölder Theorem

A **composition series** is a maximal chain: ⊥ = a₀ ≺ a₁ ≺ ... ≺ aₙ = ⊤. The **composition factors** are the simple intervals [aᵢ, aᵢ₊₁]. The Jordan-Hölder theorem states:

> In a modular lattice, any two composition series have the same length, and the composition factors are isomorphic up to permutation.

This is the lattice-theoretic analog of unique prime factorization.

### Krull-Schmidt Theorem

An element is **indecomposable** if it cannot be written as a nontrivial join. The Krull-Schmidt theorem states:

> In a modular lattice satisfying the ascending and descending chain conditions, every element can be written as a join of indecomposable elements, and this decomposition is unique up to permutation.

### Diamond Isomorphism Theorem

In a modular lattice, if a ≤ b, the interval [a, b] is isomorphic to [a ∨ x, b ∨ x] via the map y ↦ y ∨ x. This is a consequence of the modular law and generalizes the second isomorphism theorem from group theory.

## Testing

**109 tests** across 11 modules:

| Module | Tests | Coverage |
|---|---|---|
| `lattice` | 18 | Chain, diamond, pentagon, powerset, axioms, Hasse diagrams |
| `modular` | 9 | Modular law check, M₃ is modular, N₅ is not, diamond isomorphism |
| `distributive` | 10 | Distributive law check, chain/powerset are distributive |
| `boolean` | 10 | Complement laws, powerset Boolean algebra |
| `subgroup` | 10 | Cyclic groups, S₃, subgroup lattice properties |
| `subspace` | 12 | Subspace construction, lattice properties |
| `composition` | 10 | Maximal chains, Jordan-Hölder length, composition series |
| `krull_schmidt` | 10 | Indecomposable decomposition, uniqueness verification |
| `dedekind` | 11 | Pentagon detection, Dedekind theorem verification, modular identity |
| `capability` | 9 | Builder pattern, modular verification, composition guarantees |

Run with:

```bash
cargo test
```

## License

MIT
