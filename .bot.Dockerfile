FROM node
RUN wget https://sdk.dfinity.org/install.sh -O /tmp/install-sdk.sh \
 && sh -c 'yes Y | DFX_VERSION=0.9.3 sh /tmp/install-sdk.sh'

WORKDIR /app
COPY ./canister_ids.json .
COPY ./dfx.json .
COPY market_maker_bot ./market_maker_bot/

CMD ["node", "./market_maker_bot/index.js", "--network", "ic"]
