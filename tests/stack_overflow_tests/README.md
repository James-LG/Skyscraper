# stack_overflow_tests

The structs in this library are large enough that the recursive parsing of XPath expressions can cause stack overflows.

This has only ever been reproduced on Windows 10 and only when building a binary in debug mode.
Notably, this cannot be reproduced using an actual test, thus this mini project was created.

This project is run automatically during a `cargo test` of the main project by way of `run_stack_overflow_tests.rs`.

A typical failure of this tests appears as follows:

```rs
---- does_this_run stdout ----
thread 'does_this_run' panicked at tests\run_stack_overflow_tests.rs:20:5:

   Compiling skyscraper_stack_overflow_tests v0.1.0 (D:\code\Skyscraper\tests\stack_overflow_tests)
    Finished dev [unoptimized + debuginfo] target(s) in 0.30s
     Running `tests\stack_overflow_tests\target\debug\skyscraper_stack_overflow_tests.exe`

thread 'main' has overflowed its stack
error: process didn't exit successfully: `tests\stack_overflow_tests\target\debug\skyscraper_stack_overflow_tests.exe` (exit code: 0xc00000fd, STATUS_STACK_OVERFLOW)

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```