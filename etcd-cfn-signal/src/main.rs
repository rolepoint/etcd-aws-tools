extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate openssl;
extern crate clap;
extern crate rusoto;
extern crate common;

mod etcd;
mod aws;

use clap::{Arg, App};

use etcd::SSLOptions;

use common::args::arg_str;
use common::etcd::add_etcd_tls_args;

fn main() {
    let app = App::new("etcd-cfn-signal")
        .version("0.1")
        .author("Graeme Coupar <graeme@rolepoint.com>")
        .about("Notifies cloudformation of etcd readiness")
        .arg(Arg::with_name("SERVER")
             .help("The etcd server to wait for")
             .required(true)
             .index(1)
        );

    let matches = add_etcd_tls_args(app).get_matches();

    let error = etcd::wait_till_healthy(
        arg_str(&matches, "SERVER"),
        SSLOptions::from_args(&matches)
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
