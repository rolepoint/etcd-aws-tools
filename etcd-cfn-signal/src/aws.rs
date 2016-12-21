extern crate hyper;
extern crate rusoto;

use std::str::FromStr;
use std::io::Read;

use hyper::Error;

use rusoto::ec2;
use rusoto::ec2::{Ec2Client, DescribeInstancesRequest, TagList};
use rusoto::cloudformation::{CloudFormationClient, SignalResourceInput};
use rusoto::{DefaultCredentialsProvider, Region};


const INSTANCE_METADATA_URL: &'static str = "http://169.254.169.254/latest/meta-data/";
const STACK_ID_TAG: &'static str = "aws:cloudformation:stack-id";
const RESOURCE_ID_TAG: &'static str = "aws:cloudformation:logical-id";


/// Signals cloudformation that we are ready.
///
/// Panics on failure.
pub fn signal_cfn() {
    let region = get_region().expect("Getting region");

    let client = CloudFormationClient::new(
        DefaultCredentialsProvider::new().unwrap(),
        region
    );

    let instance_id = fetch_instance_metadata(
        "instance-id"
    ).expect("Getting instance ID");

    let (stack_name, resource_id) = fetch_stack_and_resource_ids(
        region, &instance_id
    ).expect("Fetching cloudformation stack info");

    println!(
        "Signaling SUCCESS for '{}' on instance '{}' in stack '{}'",
        resource_id, instance_id, stack_name
    );

    client.signal_resource(
        &SignalResourceInput{
            logical_resource_id: resource_id,
            stack_name: stack_name,
            status: "SUCCESS".to_string(),
            unique_id: instance_id
        }
    ).expect("Signalling cloudformation");
}


/// Gets the current instances region using the ec2 instance metadata service.
fn get_region() -> Result<Region, String> {
    let mut availability_zone = try!(
        fetch_instance_metadata("placement/availability-zone")
            .map_err(|_| String::from("Failed getting avaliabilty zone"))
    );

    // Removing the last char of the availability zone gives us the region.
    availability_zone.pop();

    return Region::from_str(availability_zone.as_str())
        .map_err(|_| String::from("Failed converting the region"));
}


/// Fetches instance metadata from the provided endpoint.
///
/// Endpoint should be relative to the latest/meta-data folder on the instance
/// metadata server
fn fetch_instance_metadata(endpoint : &str) -> Result<String, Error> {
    let client = hyper::Client::new();

    let url = String::from(INSTANCE_METADATA_URL) + endpoint;
    let mut resp = try!(client.get(&url).send());

    let mut data = String::new();

    resp.read_to_string(&mut data).expect("Reading instance metadata");

    return Ok(data);
}


/// Errors we can return from fetch_stack_and_resource_ids
#[derive(Debug)]
enum TagFetchError {
    DescribeInstancesError(ec2::DescribeInstancesError),
    NoTagsReturned,
    MissingStackName,
    MissingResourceId
}

/// Fethces the cloudformation stack name & logical resource names for the current
/// instance.
fn fetch_stack_and_resource_ids(region: Region, instance_id: &str) -> Result<(String, String), TagFetchError> {
    let client = Ec2Client::new(
        DefaultCredentialsProvider::new().unwrap(),
        region
    );

    let result = try!(
        client.describe_instances(
            &DescribeInstancesRequest{
                instance_ids: Some(vec![instance_id.to_string()]),
                ..Default::default()
            }
        )
    );

    return result.reservations.as_ref()
        .and_then(|reservations| reservations.first())
        .and_then(|reservation| reservation.instances.as_ref())
        .and_then(|instances| instances.first())
        .and_then(|instance| instance.tags.as_ref())
        .ok_or_else(|| TagFetchError::NoTagsReturned)
        .and_then(|tags| {
            let stack_name = find_tag_by_key(&tags, STACK_ID_TAG);
            let resource_id = find_tag_by_key(&tags, RESOURCE_ID_TAG);
            if stack_name.is_none() {
                return Err(TagFetchError::MissingStackName);
            }
            if resource_id.is_none() {
                return Err(TagFetchError::MissingResourceId);
            }
            return Ok((stack_name.unwrap(), resource_id.unwrap()));
        })
}


/// Finds a particular tag value in a TagDescriptionList
fn find_tag_by_key(tags: &TagList, key: &str) -> Option<String> {
    tags.iter()
        .find(|&tag| tag.key.is_some() && tag.key.as_ref().unwrap() == key)
        .and_then(|tag| tag.value.clone())
}


impl From<ec2::DescribeInstancesError> for TagFetchError {
    fn from(error: ec2::DescribeInstancesError) -> TagFetchError {
        return TagFetchError::DescribeInstancesError(error);
    }
}
