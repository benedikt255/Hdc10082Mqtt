FROM alpine:latest
WORKDIR /app

ARG execfile=target/release
ADD $execfile/Hdc10082Mqtt Hdc10082Mqtt
RUN chmod +x Hdc10082Mqtt
CMD ["./Hdc10082Mqtt"]
