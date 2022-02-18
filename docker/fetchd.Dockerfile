FROM fetchai/fetchd:0.9.0

COPY ./scripts/run-fetch-node.sh /usr/bin/run-node.sh

ENTRYPOINT /usr/bin/run-node.sh