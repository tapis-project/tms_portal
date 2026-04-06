##########################################################################
# Dockerfile for Tms Auth service
# This file must be placed in the build directory (dependencies)
#   before docker build is run.
#
#   $TAG            the tag for image identification
#
##########################################################################
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

# Add user tms_auth
RUN useradd -m tms_auth

USER tms_auth

# Just copy the jars needed
WORKDIR /home/tms_auth/app
COPY --chown=tms_auth:tms_auth ./target/release/tms_authenticator ./
RUN chmod +x ./tms_authenticator

# Server port, debug port and jmx port
EXPOSE 8080 

CMD ["./tms_authenticator"]

