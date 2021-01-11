# This Docker image will run the Xenon gRPC service.
# The 'nlesc/xenon-slurm' image can be used for integration testing.
FROM alpine:3.10

ENV VERSION="3.0.2"
ENV RELEASE="xenon-grpc-shadow-$VERSION"

RUN apk add --no-cache \
    curl \
    openjdk11-jre-headless

RUN curl -LO \
    "https://github.com/xenon-middleware/xenon-grpc/releases/download/v$VERSION/$RELEASE.tar" \
 && tar -xf "$RELEASE.tar" \
 && mv "$RELEASE" "xenon" \
 && rm "$RELEASE.tar"

EXPOSE 50051
ENTRYPOINT ["/xenon/bin/xenon-grpc"]
