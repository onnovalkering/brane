FROM alpine:3.13

ENV VERSION="0.12.1"
ENV RELEASE="juicefs-$VERSION-linux-amd64"

RUN apk add --no-cache \
    curl

RUN curl -LO "https://github.com/juicedata/juicefs/releases/download/v$VERSION/$RELEASE.tar.gz" \
 && tar -xf "$RELEASE.tar.gz" \
 && rm "$RELEASE.tar.gz"

ENTRYPOINT ["/juicefs"]
