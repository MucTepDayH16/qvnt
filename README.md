# QVNT

[![build](https://img.shields.io/github/workflow/status/MucTepDayH16/qvnt/Rust?style=for-the-badge&logo=github&label=build/tests)](https://github.com/MucTepDayH16/qvnt/actions/workflows/rust.yml)
[![rustc](https://img.shields.io/badge/rustc-1.40+-blue?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
[![crates.io](https://img.shields.io/crates/v/qvnt?style=for-the-badge&logo=hackthebox&logoColor=white)](https://crates.io/crates/qvnt)
[![docs.rs](https://img.shields.io/docsrs/qvnt?style=for-the-badge&logo=rust)](https://docs.rs/qvnt/)

### Advanced quantum computation simulator, written in *Rust*


## Features
1. Ability to simulate up to 64 qubits.
   Common machine with 4-16 Gb of RAM is able to simulate 26-28 qubits, which is enough for several study cases;
2. Set of 1- or 2-qubits operations to build your own quantum circuits;
3. Quantum operations are tested and debugged to be safe in use;
4. Circuit execution is accelerated using multithreading *Rayon* library;
5. Complex quantum registers manipulations: tensor product of two registers and aliases for qubit to simplify interaction with register.

___
## Usage

Add this lines to your *Cargo.toml* file to use __QVNT__ crate:

```toml
[dependencies]
qvnt = { version = "0.4.1", features = ["cpu"] }
```

Quantum register and operators are controlled by bitmasks.
Each *bit* in it will act on a specific *qubit*.

```rust
use qvnt::prelude::*;

//  Create quantum register with 10 qubits
let mut q_reg = QReg::new(10);
//  or with initial state, where 5th, 6th and 7th qubits are already in state |1>.
let mut q_reg = QReg::new(10).init_state(0b0011100000);

//  Create qft (Quantum Fourier Transform) operation, acting on first 5 qubits in q_reg.
let op = op::qft(0b0000011111);

//  Apply created operation
q_reg.apply(&op);

//  Measure and write first 3 qubit, which leads to collapse of q_reg wave function.
//  Measured variable will contain one of the following values:
//  0b000, 0b001, 0b010, 0b011, 0b100, 0b101, 0b110, 0b111
let measured = q_reg.measure_mask(0b0000000111);
```

You're able to use *VReg* to simplify operations definition:

```rust
use qvnt::prelude::*;

let mut q_reg = QReg::new(10);
let q = q_reg.get_vreg();

//  Crate Hadamard operator, that act on odd qubits.
let op = op::h(q[1] | q[3] | q[5] | q[7] | q[9]);
//  This is equivalent to op::h(0b0101010101);
```

___
## Implemented operations
* Pauli's *X*, *Y* & *Z* operators;
* Square and fourth root of *Z* - *S* & *T* operators;
* Phase shift operator - *phi*;
* 1-qubit rotation operators - *rx*, *ry* & *rz*;
* 2-qubits rotation operators, *aka* Ising coupling gates, - *rxx*, *ryy* & *rzz*;
* *SWAP*, *iSWAP* operators and square rooted ones;
* Quantum Fourier and Hadamard Transform;
* Universal *U1*, *U2* and *U3* operators;

__ALL__ operators have inverse versions, accessing by ```.dgr()``` method:
```rust
use qvnt::prelude::*;

let usual_op = op::s(0b1);
//  Inverse S operator
let inverse_op = op::s(0b1).dgr();
```

Also, __ALL__ these operators could be turned into controlled ones, using ```.c(...)``` method:
```rust
use qvnt::prelude::*;

let usual_op = op::x(0b001);
//  NOT gate, controlled by 2 qubits, aka CCNOT gate, aka Toffoli gate
let controlled_op = op::x(0b001).c(0b110).unwrap();
```
Controlled operation has to be unwrapped, since it could be None if its mask overlaps with the mask of operator.
For example, this code will *panic*:
```rust,should_panic,panics
use qvnt::prelude::*;
let _ = op::x(0b001).c(0b001).unwrap();
```


___
## License
Licensed under [MIT License](LICENSE.md)
