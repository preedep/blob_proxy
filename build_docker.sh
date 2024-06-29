docker build --cpuset-cpus="0,1" --platform linux/amd64 -t blob-proxy-rs:latest .
docker tag blob-proxy-rs:latest nickmsft/blob-proxy:latest
docker push nickmsft/blob-proxy:latest

