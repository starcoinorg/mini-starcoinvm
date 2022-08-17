FROM ghcr.io/cross-rs/mips-unknown-linux-musl:main

USER root

## RUN apt-get -y update \
## 	&& apt-get install -y wget \
## 	&& wget https://github.com/openssl/openssl/archive/refs/tags/openssl-3.0.5.tar.gz -O /opt/openssl-3.0.5.tar.gz \
## 	&& tar -zxvf /opt/openssl-3.0.5.tar.gz --directory /opt \
## 	&& /opt/openssl-openssl-3.0.5/Configure  --prefix=/usr/local/ssl --openssldir=/usr/local/ssl '-Wl,-rpath,$(LIBRPATH)' \
## 	&& make -C /opt/openssl-openssl-3.0.5 \
## 	&& make -C /opt/openssl-openssl-3.0.5 install \
## 	&& rm -rf /opt/openssl*

RUN apt-get -y update
RUN apt-get install -y wget
RUN wget https://github.com/openssl/openssl/archive/refs/tags/openssl-3.0.5.tar.gz -O /opt/openssl-3.0.5.tar.gz
RUN tar -zxvf /opt/openssl-3.0.5.tar.gz --directory /opt
RUN cd /opt/openssl-openssl-3.0.5 && ./config shared --prefix=/usr/local/ssl --openssldir=/usr/local/ssl '-Wl,-rpath,$(LIBRPATH)'
RUN make -C /opt/openssl-openssl-3.0.5
RUN make -C /opt/openssl-openssl-3.0.5 install
RUN rm -rf /opt/openssl*