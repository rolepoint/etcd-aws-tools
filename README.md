Etcd AWS Tools
---

Some simple command line tools for building etcd clusters on AWS.  [Inspired by
this blog post](https://crewjam.com/etcd-aws/) with some code borrowed from
[crewjam/etcd-aws](https://github.com/crewjam/etcd-aws/pull/14/files).

#### Differences from etcd-aws.

This repository aims to provide a single tool for each of the purposes etcd-aws
does in a single executable, and also to leverage the build in etcd instance of
a CoreOS machine, rather than vendoring one into a docker image.
