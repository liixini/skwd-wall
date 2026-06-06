import socket, json, time, sys, os

sock = sys.argv[1]
want = int(sys.argv[2])
timeout = float(sys.argv[3])


def rpc(s, method, params, mid):
    s.sendall((json.dumps({"method": method, "params": params, "id": mid}) + "\n").encode())
    buf = b""
    s.settimeout(10)
    while b"\n" not in buf:
        chunk = s.recv(65536)
        if not chunk:
            break
        buf += chunk
    return json.loads(buf.split(b"\n")[0])


deadline = time.time() + timeout
while time.time() < deadline and not os.path.exists(sock):
    time.sleep(0.1)

s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
while time.time() < deadline:
    try:
        s.connect(sock)
        break
    except OSError:
        time.sleep(0.1)

while time.time() < deadline:
    r = rpc(s, "wall.list", {}, 1)
    if (r.get("result") or {}).get("count", 0) >= want:
        sys.exit(0)
    time.sleep(0.3)

sys.exit(1)
