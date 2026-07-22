FROM alpine:3.22.1@sha256:4bcff63911fcb4448bd4fdacec207030997caf25e9bea4045fa6c8c44de311d1

RUN apk add --no-cache socat

EXPOSE 5657

ENTRYPOINT ["socat", "TCP-LISTEN:5657,fork,reuseaddr", "TCP:pumpkin.railway.internal:5657"]

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD nc -z 127.0.0.1 5657 || exit 1
