FROM ubuntu:21.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y mercurial cmake g++ qtbase5-dev qtbase5-private-dev qttools5-dev-tools qttools5-dev \
    qt5-style-plugins libsdl-ttf2.0-dev libsdl2-dev libsdl2-net-dev libsdl2-mixer-dev libsdl2-image-dev \
    libsdl2-ttf-dev liblua5.1-dev fpc libphysfs-dev fonts-dejavu-core ttf-wqy-zenhei \
    ghc libghc-binary-dev libghc-sandi-dev libghc-deepseq-dev libghc-hslogger-dev \
    libghc-mtl-dev libghc-network-dev libghc-parsec3-dev libghc-utf8-string-dev libghc-vector-dev libghc-random-dev \
    libghc-zlib-dev libghc-sha-dev libghc-entropy-dev libghc-regex-tdfa-dev libghc-aeson-dev libghc-yaml-dev libghc-text-dev\
    libghc-base-dev
    
RUN rm -rf /var/lib/apt/lists/*

VOLUME "/src"
WORKDIR /src

CMD cmake -DNOVIDEOREC=1 -DNOPNG=1 . && make install && ./bin/hedgewars