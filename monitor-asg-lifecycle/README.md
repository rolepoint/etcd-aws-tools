monitor-asg-lifecycle
---

Listends for lifecycle hooks coming in on an SQS queue for the current autoscale
group, and removes any machines from the etcd cluster.

Usage
----

Should be run from the command line.

    ./monitor-asg-lifecycle

Alternatively it can be run inside docker:

    docker run obmarg/monitor-asg-lifecycle

Development
----

Binaries can be built with

    `make all`

Docker image can be built with:

    `make docker`

Clean up old builds with:

    `make clean`
