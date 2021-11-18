# pre

Fetch Network Proxy Reencryption Service

## testing

```bash
# deploy local fetch and ipfs nodes
docker-compose -f docker/docker-compose.yml up
# run tests (on another terminal)
docker-compose -f docker/docker-compose.yml run tests
# stop all containers
docker-compose -f docker/docker-compose.yml down
```


## development

### linting and code checks

Run `make lint` to format code, arrange imports, perform mypy, pylint, flake8 checks