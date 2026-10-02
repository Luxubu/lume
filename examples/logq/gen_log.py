# Writes an access log: `python3 gen_log.py N > file`. The same N gives the
# same log, so the sample and the benchmark's big log are reproducible.
import random, sys
n = int(sys.argv[1]) if len(sys.argv) > 1 else 40
r = random.Random(7)
paths = ["/", "/api/users", "/api/orders", "/api/orders/42", "/login", "/static/app.js", "/api/search"]
weights = [5, 8, 6, 3, 2, 9, 4]
ips = ["203.0.113.%d" % i for i in range(1, 9)]
t = 1759226400  # 2025-09-30T10:00:00Z
import datetime
for i in range(n):
    t += r.choice([0, 0, 1, 1, 2, 3, 7])
    p = r.choices(paths, weights)[0]
    m = "POST" if p == "/login" or (p == "/api/orders" and r.random() < 0.3) else "GET"
    s = r.choices([200, 201, 304, 404, 500, 503], [80, 4, 8, 4, 3, 1])[0]
    if m == "POST" and s == 200: s = 201
    ms = int(r.lognormvariate(3, 0.8)) + (400 if p == "/api/search" and r.random() < 0.3 else 0)
    ts = datetime.datetime.fromtimestamp(t, datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    line = "%s %s %s %s %d %dms" % (ts, r.choice(ips), m, p, s, ms)
    if n <= 100 and i in (11, 29): line = "-- log rotated --" if i == 11 else line.replace("ms", "")
    print(line)
