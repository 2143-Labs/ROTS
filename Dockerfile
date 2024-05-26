FROM --platform=linux/amd64 debian:12-slim as runner

#RUN apt-get update
#RUN apt-get install -y bash ca-certificates curl

RUN mkdir -p /rots
WORKDIR /rots

RUN adduser rots
USER rots

COPY --chown=app:app ./bin/server /app/server
CMD ["bash", "-c", "echo starting server; /app/server"]
