FROM archlinux:base-devel

RUN pacman -Sy --noconfirm alsa-lib

WORKDIR /jolteon

COPY rust-toolchain.toml rustfmt.toml .clippy.toml Cargo.toml Cargo.lock .

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup.sh
RUN chmod a+x rustup.sh
RUN ./rustup.sh -y

ENV PATH="$HOME/.cargo/bin:$PATH"
ENV PATH="/root/.cargo/bin:$PATH"

RUN rustup target add x86_64-unknown-linux-gnu
RUN rustc --target=x86_64-unknown-linux-gnu --print target-cpus
RUN rustc --print target-list

COPY src src
COPY assets assets

RUN cargo build --locked --release --target=x86_64-unknown-linux-gnu

CMD /bin/sh