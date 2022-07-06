FROM ipfs/go-ipfs:v0.7.0


ARG ORIGIN='["https://pre-ui.web.app","https://app.dabbaflow.opencolearn.com","https://dabbaflow-app-prod.web.app","https://dabbaflow-app.opencolearn.com"]'
ENV ALLOW_ORIGIN=$ORIGIN

COPY run.sh .
COPY shutdown.sh .
COPY start_ipfs /usr/local/bin/start_ipfs

RUN  chmod +x /usr/local/bin/start_ipfs && chmod +x run.sh && ./run.sh 