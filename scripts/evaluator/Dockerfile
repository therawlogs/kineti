FROM ubuntu:24.04
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends sudo util-linux coreutils libgcc-s1 ca-certificates && rm -rf /var/lib/apt/lists/*
RUN groupadd -g 10001 evaluator && useradd -u 10001 -g 10001 -M evaluator && groupadd -g 10002 candidate && useradd -u 10002 -g 10002 -M candidate
COPY .kineti/implementation/kineti-cli /usr/local/bin/kineti-cli
COPY scripts/evaluator/candidate /usr/local/libexec/kineti-candidate
COPY scripts/evaluator/entrypoint /usr/local/bin/kineti-evaluator-entrypoint
RUN chmod 0755 /usr/local/bin/kineti-cli /usr/local/libexec/kineti-candidate /usr/local/bin/kineti-evaluator-entrypoint && printf 'evaluator ALL=(root) NOPASSWD: /usr/local/libexec/kineti-candidate\n' > /etc/sudoers.d/kineti-evaluator && chmod 0440 /etc/sudoers.d/kineti-evaluator && mkdir -p /var/lib/kineti /var/lib/kineti-jobs
ENTRYPOINT ["/usr/local/bin/kineti-evaluator-entrypoint"]
