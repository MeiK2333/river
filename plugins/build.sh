#!/bin/sh

echo "Hello World!"
# create judge dir
mkdir -p /river/runner

cd /plugins/js
npm install
npm install -g ts-node typescript

echo "Hello World!"
