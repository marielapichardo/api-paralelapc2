# 1. Imagen base de Rust
FROM rust:latest AS builder

# 2. Crear un directorio de trabajo dentro del contenedor
WORKDIR /usr/src/app

# 3. Copiar el archivo Cargo.toml y Cargo.lock al contenedor
COPY Cargo.toml Cargo.lock ./

# 4. Descargar las dependencias
RUN cargo fetch

# 5. Copiar el código fuente al contenedor
COPY . .

# 6. Compilar el proyecto en modo release
RUN cargo build --release

# 7. magen más ligera para ejecutar la aplicación
FROM debian:bookworm-slim
WORKDIR /usr/local/bin

# dependencias
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    --no-install-recommends && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# 8.  Copiar el binario generado
COPY --from=builder /usr/src/app/target/release/ApiParalela /usr/local/bin/ApiParalela

# Copiar el archivo fcm_account.json al contenedor
COPY fcm_account.json /usr/local/bin/fcm_account.json

# 9. Exponer el puerto de la aplicación
EXPOSE 3030
#10  Ejecutar el binario
CMD ["ApiParalela"]