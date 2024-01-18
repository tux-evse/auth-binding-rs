#!/bin/bash

export LD_LIBRARY_PATH=/usr/local/lib64
pkill afb-auth
cynagora-admin set '' 'HELLO' '' '*' yes
clear

# build test config dirname
DIRNAME=`dirname $0`
cd $DIRNAME/..
CONFDIR=`pwd`/etc
mkdir -p /tmp/api

DEVTOOL_PORT=1238
echo auth debug mode config=$CONFDIR/*.json port=$DEVTOOL_PORT

afb-binder --name=afb-auth --port=$DEVTOOL_PORT -v \
  --config=$CONFDIR/binder-auth.json \
  --config=$CONFDIR/binding-auth.json \
  --config=$CONFDIR/binding-scard.json \
  --tracereq=all \
  $*