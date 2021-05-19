# QVNT
Advansed quantum computation simulator, written in *Rust*
___
## Features
1. Ability to simulate up to 64 qubits, which is a limit for 64-bits machines.
   But usual machine (with 4Gb RAM) only allowed to run 26 qubits, which is enough for study cases.
2. A set of necessary 1- or 2-qubits operations, including general 1x1 and 2x2 unitary matrix, to build your own quantum circuits.
3. Existed quantum operations are tested and debugged to be safe in use.
4. Accelerated circuit execution using multithreaded *Rayon* library.
5. Complex quantum registers manipulations: tensor product of two registers and aliases for qubit to *humanify* interaction with register

___
## Usage
```rust
use qvnt::{
   operator::Op,
   register::QReg,
}

//  create quantum register, called 'x', with 10 qubits
let mut q_reg = QReg::new(10).alias_char('x');
//  or with initial state, where 3 qubits are already in state |1>
//  let q_reg = QReg::new(10).alias_char('x').init_state(0b0011100000);

//  get register 'x', to interact with specified qubits
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
*  Pauli's *X*, *Y* & *Z* operators;
*  Phase shift operator;
*  1-qubit rotation operators;
*  2-qubits rotation operators, *aka* Ising gates;
*  *SWAP*, *iSWAP* operators and square rooted ones;
*  *QFT* with and without swapping qubits after applying;
*  General unitary operators, constructed from 2x2 or 4x4 complex matrices; 

Also, __ALL__ these operators could be turned into controlled ones, using ```.c(...)``` syntax:
```rust
let usual_op = Op::x(0b001);
//  NOT gate, controlled by 2 qubits, aka CCNOT gate, aka Toffoli gate
let controlled_op = Op::x(0b001).c(0b110);
```

___
## In work
1. Optimizing and vectorizing operations.
2. Adding inverse operators for implemented ones.
