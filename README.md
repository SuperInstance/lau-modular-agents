# lau-modular-agents

Modular lattice theory for agents — the algebraic structure of capability hierarchies.

## Overview

This crate provides a comprehensive implementation of modular lattice theory with applications to agent capability hierarchies. It covers:

- **Lattice**: poset with join (∨) and meet (∧)
- **Modular lattice**: if a ≤ b then a ∨ (x ∧ b) = (a ∨ x) ∧ b
- **Distributive lattice**: a ∧ (b ∨ c) = (a ∧ b) ∨ (a ∧ c) (stronger than modular)
- **Boolean algebra**: complemented distributive lattice
- **Subgroup lattice**: lattice of subgroups of an agent group
- **Lattice of subspaces**: subspaces of an agent vector space form a modular lattice
- **Jordan-Hölder theorem**: composition series length is invariant
- **Krull-Schmidt theorem**: decomposition into indecomposables is unique
- **Modular law and Dedekind's theorem**
- **Application**: capability hierarchy as modular lattice (clean composition guarantees)

## Usage

```rust
use lau_modular_agents::{chain_lattice, check_modular_law, diamond_m3_lattice};

// Chain lattices are always modular
let chain = chain_lattice(5);
assert!(check_modular_law(&chain).is_ok());

// The diamond M₃ is modular but not distributive
let m3 = diamond_m3_lattice();
assert!(check_modular_law(&m3).is_ok());
```

## Capabilities as Modular Lattices

The key insight: agent capabilities form a modular lattice where:
- **Join** = combined capability
- **Meet** = shared capability
- **Modularity** guarantees clean composition: composing capabilities preserves structure

This gives mathematical guarantees about how agent capability hierarchies compose.

## License

MIT
