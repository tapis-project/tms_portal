##########################################################################
# Dockerfile for Tms Portal service
# This file must be placed in the build directory (dependencies)
#   before docker build is run.
#
#   $TAG            the tag for image identification
#
##########################################################################

FROM node:24 AS node_build
RUN npm install -g pnpm
COPY ./client/package.json ./client/pnpm-lock.yaml ./client/pnpm-workspace.yaml /client/
WORKDIR /client
RUN pnpm install
COPY ./client/ /client/
RUN pnpm run build


FROM ubuntu:jammy

LABEL maintainer="CIC Support <cicsupport@tacc.utexas.edu>"

# apt update
RUN apt update 

# Install less. 
RUN apt-get install -y less 

# Install vi.
RUN apt-get install -y vim-tiny

# Install ca certs
RUN apt-get install -y ca-certificates

# Add user tms_portal
RUN useradd -m tms_portal

USER tms_portal

# Just copy the jars needed
WORKDIR /home/tms_portal/app
COPY --chown=tms_portal:tms_portal ./target/release/tms_portal ./
RUN mkdir ./config
COPY --chown=tms_portal:tms_portal ./application/resources/config ./config
COPY --from=node_build --chown=tms_portal:tms_portal /client/dist/ ./dist/
RUN chmod +x ./tms_portal

# Server port, debug port and jmx port
EXPOSE 8080 

CMD ["./tms_portal"]

