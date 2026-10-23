FROM ubuntu:24.04@sha256:68ee22004a9109b44973313b02ff7b1de250f1de0f5ecb628d73f579aaff33ac
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends util-linux coreutils libgcc-s1 && rm -rf /var/lib/apt/lists/*
COPY bun /usr/local/bin/bun
COPY check-spend.ts /trusted/check-spend.ts
RUN chmod 0555 /usr/local/bin/bun && chmod 0700 /trusted && chmod 0400 /trusted/check-spend.ts
ENTRYPOINT ["/usr/local/bin/bun","/trusted/check-spend.ts"]
