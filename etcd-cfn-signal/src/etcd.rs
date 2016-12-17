extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate openssl;

use std::time::Duration;
use std::thread;

use hyper::Error;
use hyper::status::StatusCode;

use serde_json::Value;

pub use common::etcd::SSLOptions;
use common::etcd::etcd_https_client;

/// Polls the etcd health endpoint until it reports we're healthy.
pub fn wait_till_healthy(server_url: String, options: SSLOptions)
                         -> Result<(), Error> {
    let client = try!(etcd_https_client(options));
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
