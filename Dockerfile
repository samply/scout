FROM node AS tailwind
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .

# Build the Tailwind CSS file
RUN npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --minify

FROM rust:1 AS builder
WORKDIR /usr/src/app

# Install `dx`
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli --root /.cargo -y --force
ENV PATH="/.cargo/bin:$PATH"

# Copy the source code and Tailwind CSS file
COPY . .
COPY --from=tailwind /app/assets/tailwind.css assets/

# Create the final bundle folder. Bundle always executes in release mode with optimizations enabled
RUN --mount=type=cache,target=/usr/src/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    dx bundle --platform web --out-dir bundle

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /usr/src/app/bundle .
COPY codesystems codesystems

# Set our port and make sure to listen for all connections
ENV PORT=8080
ENV IP=0.0.0.0

# Expose the port 8080
EXPOSE 8080

ENTRYPOINT ["./server"]
