#!/bin/sh

eval `ec2cluster`;
cfn-signal --resource $1 --stack $TAG_AWS_CLOUDFORMATION_STACK_ID --region $REGION;
