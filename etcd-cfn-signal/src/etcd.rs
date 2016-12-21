extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate openssl;

use std::sync::Arc;
use std::time::Duration;
use std::thread;

use hyper::{Client, Error};
use hyper::net::{HttpsConnector, Openssl};
use hyper::status::StatusCode;

use openssl::ssl::{SslContext, SslMethod};
use openssl::x509::X509FileType;

use serde_json::Value;

pub struct SSLOptions {
    pub ca_file: Option<String>,
    pub cert_and_key: Option<(String, String)>
}

/// Polls the etcd health endpoint until it reports we're healthy.
pub fn wait_till_healthy(server_url: String, options: SSLOptions)
                         -> Result<(), Error> {
    let client = try!(etcd_client(options));
    let health_url = server_url + "/health";

    loop {
        thread::sleep(Duration::from_millis(1000));
        println!("Checking etcd status...");

        if let Ok(response) = client.get(&health_url).send() {
            if response.status != StatusCode::Ok {
                println!("Got HTTP {}", response.status);
                continue;
            }

            let value: Value = serde_json::from_reader(response).unwrap();
            let health = value
                .as_object().unwrap()
                .get("health").unwrap()
                .as_str().unwrap();

            if health == "true" {
                return Ok(());
            } else {
                println!("Health is {}", health);
            }
        } else {
            println!("Etcd not responding.  Will try again");
        }
    }
}


/// Creates an HTTPS client for talking to etcd.
fn etcd_client (options: SSLOptions) -> Result<Client, Error> {
    let mut ctx = try!(SslContext::new(SslMethod::Sslv23));

    if let Some(ref ca) = options.ca_file {
        try!(ctx.set_CA_file(ca));
    }

    if let Some((ref cert, ref key)) = options.cert_and_key {
        try!(ctx.set_certificate_file(cert, X509FileType::PEM));
        try!(ctx.set_private_key_file(key, X509FileType::PEM));
    }

    return Ok(Client::with_connector(
        HttpsConnector::new(
            Openssl { context: Arc::new(ctx) }
        )
    ));
}

