# rust (non-alpine) image
FROM rust:1.42 as app
WORKDIR /usr/local/src
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y apt-utils busybox curl docker.io git supervisor

WORKDIR /usr/local/src

RUN mkdir -p /usr/local/src/pki/any
COPY --from=app /usr/local/cargo/bin/deploybot /usr/local/bin/deploybot
COPY --from=app /usr/local/src/config/pki/any /usr/local/src/pki/any

# Install google cloud SDK
# resource: https://stackoverflow.com/questions/43571787/docker-not-found-with-dockerdind-google-cloud-sdk
RUN curl https://sdk.cloud.google.com | bash -s -- --disable-prompts

ENV PATH "${PATH}:/root/google-cloud-sdk/bin"

# Init google cloud SDK and authenticate using service account credentials
# resource: https://github.com/GoogleCloudPlatform/cloud-sdk-docker/blob/master/alpine/Dockerfile

RUN gcloud config set core/disable_usage_reporting true && \
    gcloud config set component_manager/disable_update_check true && \
    gcloud config set metrics/environment github_docker_image && \
    gcloud components install docker-credential-gcr && \
    gcloud auth configure-docker && \
    gcloud components install kubectl && \
    gcloud --version

CMD ["bash"]
