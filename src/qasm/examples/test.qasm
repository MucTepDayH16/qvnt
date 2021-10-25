OPENQASM 2.0;

// Clifford gate: Hadamard
// gate h a { u2(0,pi) a; }

qreg q[2];
creg c[2];

h q[0];
cx q[0], q[1];
rx(3.141592653589793) q[0];

measure q -> c;