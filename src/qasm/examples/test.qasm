OPENQASM 2.0;

// Clifford gate: Hadamard
// gate h a { u2(0,pi) a; }

qreg q[2];
creg c[2];

gate foo(x, y) a, b {
    rx(x) a;
}

h q[0];
cx q[0], q[1];
foo(3.141592653589793, 0) q[0], q[1];
