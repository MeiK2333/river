#!/bin/sh

echo "Hello World!"
# create judge dir
mkdir -p /river/runner

# add to path
export PATH=$PATH:/plugins/js
cd /plugins/js
npm install

echo "Hello World!"
