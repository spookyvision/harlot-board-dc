ARG VARIANT=bullseye
FROM debian:${VARIANT}
ENV DEBIAN_FRONTEND=noninteractive
ENV LC_ALL=C.UTF-8
ENV LANG=C.UTF-8

# Arguments
ARG CONTAINER_USER=esp
ARG CONTAINER_GROUP=esp
ARG TOOLCHAIN_VERSION=1.61.0.0
ARG ESP_IDF_VERSION=release/v4.4
ARG ESP_BOARD=esp32
ARG INSTALL_RUST_TOOLCHAIN=install-rust-toolchain.sh

# Install dependencies
RUN apt-get update \
    && apt-get install -y git curl gcc clang ninja-build libudev-dev unzip xz-utils\
    python3 python3-pip python3-venv libusb-1.0-0 libssl-dev pkg-config libtinfo5 libpython2.7 \
    && apt-get clean -y && rm -rf /var/lib/apt/lists/* /tmp/library-scripts

# Set users
RUN adduser --disabled-password --gecos "" ${CONTAINER_USER}
USER ${CONTAINER_USER}
WORKDIR /home/${CONTAINER_USER}

# Install Rust toolchain, extra crates and esp-idf
ENV PATH=${PATH}:/home/${CONTAINER_USER}/.cargo/bin:/home/${CONTAINER_USER}/opt/bin

ADD --chown=${CONTAINER_USER}:${CONTAINER_GROUP} \
    https://github.com/esp-rs/rust-build/releases/download/v${TOOLCHAIN_VERSION}/${INSTALL_RUST_TOOLCHAIN} \
    /home/${CONTAINER_USER}/${INSTALL_RUST_TOOLCHAIN}

RUN chmod a+x ${INSTALL_RUST_TOOLCHAIN} \
    && ./${INSTALL_RUST_TOOLCHAIN} \
    --extra-crates "ldproxy cargo-espflash" \
    --clear-cache "YES" --export-file /home/${CONTAINER_USER}/export-esp.sh \
    --esp-idf-version "${ESP_IDF_VERSION}" \
    --minified-esp-idf "YES" \
    --build-target "${ESP_BOARD}"

# Install web-flash and wokwi-server
RUN curl -L https://github.com/bjoernQ/esp-web-flash-server/releases/latest/download/web-flash-x86_64-unknown-linux-gnu.zip \
    -o /home/${CONTAINER_USER}/.cargo/bin/web-flash.zip \
    && unzip /home/${CONTAINER_USER}/.cargo/bin/web-flash.zip \
    -d /home/${CONTAINER_USER}/.cargo/bin/ \
    && rm /home/${CONTAINER_USER}/.cargo/bin/web-flash.zip
RUN chmod u+x /home/${CONTAINER_USER}/.cargo/bin/web-flash
RUN curl -L https://github.com/MabezDev/wokwi-server/releases/latest/download/wokwi-server-x86_64-unknown-linux-gnu.zip \
    -o /home/${CONTAINER_USER}/.cargo/bin/wokwi-server.zip \
    && unzip /home/${CONTAINER_USER}/.cargo/bin/wokwi-server.zip \
    -d /home/${CONTAINER_USER}/.cargo/bin/ \
    && rm /home/${CONTAINER_USER}/.cargo/bin/wokwi-server.zip
RUN chmod u+x /home/${CONTAINER_USER}/.cargo/bin/wokwi-server
RUN rustup +nightly-x86_64-unknown-linux-gnu component add clippy
RUN rustup +nightly-x86_64-unknown-linux-gnu component add rustfmt

# Activate ESP environment
RUN echo "source /home/${CONTAINER_USER}/export-esp.sh" >> ~/.bashrc

CMD [ "/bin/bash" ]