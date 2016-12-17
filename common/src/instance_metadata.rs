extern crate hyper;
extern crate rusoto;

use std::str::FromStr;
use std::io::Read;

use rusoto::Region;

const INSTANCE_METADATA_URL: &'static str = "http://169.254.169.254/latest/meta-data/";

// Error type for fetching instance metadata.
#[derive(Debug)]
pub enum InstanceMetadataError {
    HttpError(hyper::Error),
    ParseRegionError(rusoto::ParseRegionError)
}

pub fn instance_id() -> Result<String, InstanceMetadataError> {
    return fetch_instance_metadata("instance-id");
}

/// Gets the current instances region using the ec2 instance metadata service.
pub fn region() -> Result<Region, InstanceMetadataError> {
    let mut availability_zone = try!(
        fetch_instance_metadata("placement/availability-zone")
    );

    // Removing the last char of the availability zone gives us the region.
    availability_zone.pop();

    return Region::from_str(availability_zone.as_str())
        .map_err(InstanceMetadataError::ParseRegionError);
}


/// Fetches instance metadata from the provided endpoint.
///
/// Endpoint should be relative to the latest/meta-data folder on the instance
/// metadata server
fn fetch_instance_metadata(endpoint : &str) -> Result<String, InstanceMetadataError> {
    let client = hyper::Client::new();

    let url = String::from(INSTANCE_METADATA_URL) + endpoint;
    return client.get(&url).send()
        .map_err(InstanceMetadataError::HttpError)
        .and_then(|mut resp| {
            let mut data = String::new();
            resp.read_to_string(&mut data).expect("Reading instance metatadata");
            Ok(data)
        });
}
