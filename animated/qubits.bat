@echo off
for %%f in (X X0 X1 Y Y0 Y1 Z Z0 Z1 H H0 H1) do (
    .\.venv\Scripts\manim.exe --format=gif .\qubits.py %%f
)