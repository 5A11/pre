FROM fetchai/fetchd:0.10.4

COPY ./scripts/run-fetch-node.sh /usr/bin/run-node.sh

ENTRYPOINT /usr/bin/run-node.sh