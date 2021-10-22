OPENQASM 2.0;

// Clifford gate: Hadamard
// gate h a { u2(0,pi) a; }

qreg q[2];
creg c[2];

h q[0];
CX q[0], q[1];

measure q -> c;