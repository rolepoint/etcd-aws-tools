extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate openssl;
extern crate clap;
extern crate rusoto;

mod etcd;
mod aws;

use clap::{Arg, App, ArgMatches};

use etcd::SSLOptions;

fn main() {
    let matches = App::new("etcd-cfn-signal")
        .version("0.1")
        .author("Graeme Coupar <graeme@rolepoint.com>")
        .about("Notifies cloudformation of etcd readiness")
        .arg(Arg::with_name("ca")
             .long("ca")
             .value_name("FILE")
             .help("Sets the ca certificate for etcd tls")
             .takes_value(true)
             .required(true)
             .validator(file_exists)
        )
        .arg(Arg::with_name("key")
             .long("key")
             .value_name("FILE")
             .help("Sets the private key for etcd tls")
             .takes_value(true)
             .required(true)
             .validator(file_exists)
        )
        .arg(Arg::with_name("cert")
             .long("cert")
             .value_name("FILE")
             .help("Sets the ca certificate for etcd tls")
             .takes_value(true)
             .required(true)
             .validator(file_exists)
        )
        .arg(Arg::with_name("SERVER")
             .help("The etcd server to wait for")
             .required(true)
             .index(1)
        )
        .get_matches();

    let error = etcd::wait_till_healthy(
        arg_str(&matches, "SERVER"),
        SSLOptions {
            ca_file: Some(arg_str(&matches, "ca")),
            cert_and_key: Some((
                arg_str(&matches, "cert"),
                arg_str(&matches, "key")
            ))
        }
    );
    match error {
        Ok(_) => {
            println!("etcd is healthy!");
            println!("Signaling cloudformation...");
            aws::signal_cfn();
            println!("We're good!");
        },
        Err(error) => {
            println!("Oh no: {}", error);
        },
    }
}

/// Gets the argument as a string from matches.
///
/// Argument _must_ be required or we'll crash.
fn arg_str<S: AsRef<str>>(matches: &ArgMatches, arg_name: S) -> String {
    return matches.value_of(arg_name).unwrap().to_string();
}


fn file_exists(file: String) -> Result<(), String> {
    let path = std::path::Path::new(&file);
    if !path.exists() {
        return Err(format!("File '{}' does not exist", file))
    }
    if !path.is_file() {
        return Err(format!("'{}' is not a file", file));
    }
    return Ok(());
}
