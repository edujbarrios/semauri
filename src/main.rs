// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = semauri::run_cli(&args);

    print!("{}", result.stdout);
    eprint!("{}", result.stderr);
    std::process::exit(result.status);
}
