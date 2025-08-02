docker build . -t cardinal-orcas -f .\docker\Dockerfile.aarch64
docker rm cardinal-orcas
docker create --name cardinal-orcas cardinal-orcas
docker cp cardinal-orcas:/cardinal-orcas.aarch64.bin .
docker cp cardinal-orcas:/cardinal-demo.aarch64.bin .
