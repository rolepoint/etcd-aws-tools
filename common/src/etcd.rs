extern crate std;

use std::sync::Arc;

use hyper::{Client, Error};
use hyper::net::{HttpsConnector, Openssl};

use openssl::ssl::{SslContext, SslMethod};
use openssl::x509::X509FileType;

use clap::{App, Arg, ArgMatches};

use args::arg_str;


/// Adds etcd TLS certificate arguments to the provided app.
pub fn add_etcd_tls_args<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    return app
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
}


pub struct SSLOptions {
    pub ca_file: Option<String>,
    pub cert_and_key: Option<(String, String)>
}


impl SSLOptions {
    /// Parses some SSLOptions out of arguments.
    ///
    /// It is assumed that the arguments came from an app that was passed to
    /// add_etcd_tls_args
    pub fn from_args(args: &ArgMatches) -> SSLOptions {
        return SSLOptions{
            ca_file: Some(arg_str(args, "ca")),
            cert_and_key: Some(
                (arg_str(args, "cert"), arg_str(args, "key")
                )
            )
        }
    }
}


/// Creates an HTTPS client for talking to etcd.
pub fn etcd_https_client (options: SSLOptions) -> Result<Client, Error> {
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
