FROM rust:latest
RUN /bin/bash -c 'apt-get update'
RUN /bin/bash -c 'apt-get -y upgrade'
WORKDIR /usr/src/ticker_04
COPY . .
RUN mkdir /var/log/ticker_04
RUN cargo install --path .
# CMD coinbase_client_01 2>&1 | tee -a /log/crypto_mon_$(date '+%Y%m%d%S').log
CMD ticker_04

# docker build -t ticker_04 .
#	docker run -d \
#	 --restart always \
#	 --name coinbase_client_01 \
#	-e COINBASE_URL=wss://ws-feed-public.sandbox.pro.coinbase.com \
#	-e COIN_DATABASE_URL=postgres://postgres:PASSWORD@10.1.1.205:54320/coin \
#	-e COINBASE_SANDBOX=true \
#   -e RUST_LOG=info \
#	  coinbase_client_01:latest
#	docker run -d \
#	 --restart always \
#	 --name coinbase_client_sandbox \
#	-e COINBASE_URL=wss://ws-feed-public.sandbox.pro.coinbase.com \
#	-e COIN_DATABASE_URL=postgres://postgres:PASSWORD@10.1.1.205:54320/coin \
#	-e COINBASE_SANDBOX=true \
#	-e RUST_LOG=info \
#	  coinbase_client_01:latest