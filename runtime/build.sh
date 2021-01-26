docker build . -t river
docker run --name river river
rm -rf rootfs
docker cp river:/ ./rootfs
docker stop river
docker rm river

chmod -R 755 rootfs

mknod -m 0666 rootfs/dev/null c 1 3
