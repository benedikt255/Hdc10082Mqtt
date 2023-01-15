FROM debian:stable-slim
WORKDIR /app

RUN apt-get update && \
    apt-get install -y libi2c0 i2c-tools

ARG execfile=target/release
ADD $execfile/Hdc10082Mqtt Hdc10082Mqtt
RUN chmod +x Hdc10082Mqtt
CMD ["./Hdc10082Mqtt"]
