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
    
RUN apt-get install -y mesa-utils

RUN rm -rf /var/lib/apt/lists/*

# nvidia-docker hooks
LABEL com.nvidia.volumes.needed="nvidia_driver"
ENV PATH /usr/local/nvidia/bin:${PATH}
ENV LD_LIBRARY_PATH /usr/local/nvidia/lib:/usr/local/nvidia/lib64:${LD_LIBRARY_PATH}

VOLUME "/src"
WORKDIR /src

VOLUME "/.hedgewars"

CMD cp -r /.hedgewars /root/.hedgewars && cmake -DCMAKE_BUILD_TYPE="Release" -DNOSERVER=0 -DNOVIDEOREC=1 -DNOPNG=1 -OFFICIAL_SERVER=1 . && make install && ./bin/hedgewars