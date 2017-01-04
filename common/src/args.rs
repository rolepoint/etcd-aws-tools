extern crate std;
extern crate clap;

use clap::ArgMatches;

/// Gets the argument as a string from matches.
///
/// Argument _must_ be required or we'll crash.
pub fn arg_str<S: AsRef<str>>(matches: &ArgMatches, arg_name: S) -> String {
    return matches.value_of(arg_name).unwrap().to_string();
}

