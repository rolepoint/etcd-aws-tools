extern crate hyper;
extern crate openssl;
extern crate clap;
extern crate common;

use std::str::FromStr;

use clap::{Arg, App};

use hyper::server::{Server, Request, Response, Handler};
use hyper::status::StatusCode;

use common::args::arg_str;
use common::etcd::{SSLOptions, etcd_https_client, add_etcd_tls_args};


fn main() {
    let app = App::new("health-proxy")
        .version("0.1")
        .author("Graeme Coupar <graeme@rolepoint.com>")
        .about("Proxies the local etcd health endpoint.")
        .arg(Arg::with_name("SERVER")
             .help("The etcd server to talk to")
             .required(true)
             .index(1)
        )
        .arg(Arg::with_name("port")
             .short("p")
             .long("port")
             .value_name("PORT")
             .help("The port to listen on")
             .takes_value(true)
             .required(true)
             .validator(is_number)
        );

    let matches = add_etcd_tls_args(app).get_matches();
    let client = etcd_https_client(
        SSLOptions::from_args(&matches)
    ).expect("Getting etcd client");
    let health_url = arg_str(&matches, "SERVER") + "/health";
    let port = u16::from_str(&arg_str(&matches, "port")).unwrap();

    Server::http(("0.0.0.0", port))
        .expect("Starting Server")
        .handle(HealthProxyHandler{
            client: client,
            health_url: health_url
        }).expect("Added handler");
}


struct HealthProxyHandler {
    client: hyper::Client,
    health_url: String
}


impl Handler for HealthProxyHandler {
    fn handle(&self, _request: Request, mut response: Response) {
        match self.client.get(&self.health_url).send() {
            Ok(mut etcd_response) => {
                *response.status_mut() = etcd_response.status;
                *response.headers_mut() = etcd_response.headers.clone();
                std::io::copy(
                    &mut etcd_response,
                    &mut response.start().unwrap()
                ).unwrap();
            }
            Err(_) =>
                *response.status_mut() = StatusCode::InternalServerError,
        }
    }
}


fn is_number(num: String) -> Result<(), String> {
    match u16::from_str(&num) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
