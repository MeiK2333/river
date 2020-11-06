#!/bin/sh

echo "Hello World!"
# create judge dir
mkdir -p /river/runner
# 运行的上层目录创建 node_modules，以便 Node 与 TypeScript 使用
cd /river/runner
npm i

cd /plugins/js
npm install
npm install -g ts-node typescript

echo "Hello World!"
