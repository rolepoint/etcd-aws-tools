gen-etcd-discovery-params
---

A command line tool that determines what INITIAL_CLUSTER parameters should be
passed to etcd based on the current autoscale group.

Usage
----

Should be run from the command line.

    ./gen-etcd-discovery-params

Output is designed to be piped into a config environment file for etcd:

    ./gen-etcd-discovery-params > etcd.env

Alternatively it can be run inside docker:

    docker run obmarg/gen-etcd-discovery-params

Development
----

Binaries can be built with

    `make all`

Docker image can be built with:

    `make docker`

Clean up old builds with:

    `make clean`
