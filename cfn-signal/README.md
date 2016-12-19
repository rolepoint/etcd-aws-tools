cfn-signal
----

Builds a docker image that provides a wrapper around the AWS tool cfn-signal.
This can be used to tell cloudformation that the machine has finished starting
up.  This allows cloudformation to do rolling updates of the cluster:  it only
takes down a machine after a replacement has started up and reported it's
readiness with cfn-signal.

We make use of the ec2cluster go utility to get environment varaibles
describing the current ec2cluster.
