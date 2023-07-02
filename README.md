# BI-PSI

This is my solution for the semestral project
in BI-PSI Computer networks subject
at FIT CTU in Prague in 2022/2023.

This was my first code written in Rust, so please be kind to it.

The assignment was to implement simple protocol (server side)
where a robot solves a maze on Mars, we send commands and
receive (new) location. The communication starts with
the client logging in, than path finding continues until [0, 0] is found.
The communication can be interrupted at any time
as the robot needs to recharge from time to time.
The server must be able to handle parallel requests.

