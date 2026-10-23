import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
/** Fixed, non-model program used only to test the host's network sandbox. */
export function prepareNetworkProbe(work: string) {
    fs.mkdirSync(work, { recursive: true, mode: 0o700 });
    const source = path.join(work, "network-probe.c"), binary = path.join(work, "network-probe");
    if (fs.existsSync(source) || fs.existsSync(binary))
        throw new Error("network capability probe must use a fresh host directory");
    fs.writeFileSync(source, '#include <sys/socket.h>\n#include <arpa/inet.h>\n#include <errno.h>\n#include <stdlib.h>\n#include <unistd.h>\nint main(int n,char**v){if(n!=2)return 3;int p=atoi(v[1]);if(p<1||p>65535)return 3;int s=socket(AF_INET,SOCK_STREAM,0);if(s<0)return (errno==1||errno==13||errno==97)?0:3;struct sockaddr_in a={0};a.sin_family=AF_INET;a.sin_port=htons(p);a.sin_addr.s_addr=htonl(INADDR_LOOPBACK);alarm(2);if(connect(s,(struct sockaddr*)&a,sizeof a)==0)return 1;return (errno==1||errno==13||errno==101||errno==111)?0:3;}\n', { mode: 0o600, flag: "wx" });
    const r = spawnSync("cc", ["-O2", "-o", binary, source], { encoding: "utf8", timeout: 15000 });
    if (r.status !== 0)
        throw new Error("a trusted C compiler is required for native network capability tests");
    fs.chmodSync(binary, 0o500);
    return binary;
}
