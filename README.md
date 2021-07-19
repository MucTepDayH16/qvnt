# QVNT

![minimum supported rust version](https://img.shields.io/badge/rustc-1.40+-red.svg)
[![Crate](https://img.shields.io/crates/v/qvnt.svg)](https://crates.io/crates/qvnt)
[![docs.rs](https://docs.rs/qvnt/badge.svg)](https://docs.rs/qvnt/)

Advanced quantum computation simulator, written in *Rust*


___
## Features
1. Ability to simulate up to 64 qubits.
   Common machine with 4-16 Gb of RAM is able to simulate 26-28 qubits, which is enough for several study cases;
2. Required set of 1- or 2-qubits operations to build your own quantum circuits;
3. Quantum operations are tested and debugged to be safe in use;
4. Circuit execution is accelerated using multithreading *Rayon* library;
5. Complex quantum registers manipulations: tensor product of two registers and aliases for qubit to simplify interaction with register.

___
## Usage
```rust
use qvnt::prelude::*;

//  create quantum register, called 'x', with 10 qubits
let mut q_reg = QReg::new(10).alias_char('x');
//  or with initial state, where 3 qubits are already in state |1>
//  let q_reg = QReg::new(10).alias_char('x').init_state(0b0011100000);

//  get virtual register 'x', to interact with specified qubits
let x = q_reg.get_vreg_by_char('x').unwrap();

//  create qft operation, acting on first 5 qubits in q_reg
let op = Op::qft(x[0] | x[1] | x[2] | x[3] | x[4]);

//  apply operation
q_reg.apply(&op);

//  measure and write first 3 qubit, which leads to collapse of q_reg wave function
println!("{}", q_reg.measure_mask(x[0] | x[1] | x[2]));
```

___
## Implemented operations
* Pauli's *X*, *Y* & *Z* operators;
* *S* & *T* operators;
* Phase shift operator;
* 1-qubit rotation operators;
* 2-qubits rotation operators, *aka* Ising coupling gates;
* *SWAP*, *iSWAP* operators and square rooted ones;
* Quantum Fourier and Hadamard Transform;
* Universal *U1*, *U2* and *U3* operators;

__ALL__ operators have inverse versions, accessing by ```.dgr()``` method:
```rust
let usual_op = op::s(0b1);
//  Inverse S operator
let inverse_op = op::s(0b1).dgr();
```

Also, __ALL__ these operators could be turned into controlled ones, using ```.c(...)``` method:
```rust
let usual_op = op::x(0b001);
//  NOT gate, controlled by 2 qubits, aka CCNOT gate, aka Toffoli gate
let controlled_op = op::x(0b001).c(0b110);
```

___
## In work
1. Optimizing and vectorizing operations.
3. Writing documentation for all modules.
