# Planned features and goals

Not many.

## Smaller features

- Return a non-zero error if code throws; opt-in
- Stop evaluating forms after a throw; opt-in
- Write execution log to a file; opt-in

## Larger features

- `^C` interrupts evaluation (now: runaway)
- Reuse tunneled connection (see OpenSSH ControlMaster)
- Tooling for renedering script docstrings
- Proper argument handling (now: text replacement)
- Some kind of "are you sure?" confirmation mechanism (production ops)
