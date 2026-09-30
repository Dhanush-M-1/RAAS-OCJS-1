#include <bits/stdc++.h>
using std::sort;
using std::swap;
using std::unique;
namespace fastIO {
static char buf[(1 << 19)], *p1 = buf + (1 << 19), *pend = buf + (1 << 19);
inline char nc() {
  if (p1 == pend) {
    p1 = buf;
    pend = buf + fread(buf, 1, (1 << 19), stdin);
  }
  return *p1++;
}
inline long long read() {
  long long x = 0, f = 1;
  register char s = nc();
  for (; !isdigit(s); s = nc())
    if (s == '-') f = -1;
  for (; isdigit(s); s = nc()) x = (x << 1) + (x << 3) + s - '0';
  return x * f;
}
}  // namespace fastIO
using namespace fastIO;
inline long long mabs(long long x) { return x > 0 ? x : -x; }
long long gcd(long long a, long long b) { return b ? gcd(b, a % b) : a; }
struct IT {
  long long p;
  long long k;
  IT(long long p = 0, long long k = 0) : p(p), k(k) {}
};
struct rec {
  IT p[1001];
  long long num;
  void init(long long x) {
    for (long long i = 2; i * i <= x; ++i) {
      if (x % i == 0) {
        long long c = 0;
        while (x % i == 0) x /= i, c++;
        p[num++] = IT(i, c);
      }
    }
    if (x > 1) p[num++] = IT(x, 1);
  }
  IT query(long long x) {
    for (long long i = 0; i < num; ++i)
      if (p[i].p == x) return p[i];
    return IT(x, 0);
  }
} A[200], B[200];
long long n;
IT A1[3507], B1[3507], A2[3507], B2[3507];
long long pri[3507], num;
long long exgcd(long long a, long long b, long long &x, long long &y) {
  if (!b) {
    x = 1, y = 0;
    return a;
  }
  long long g = exgcd(b, a % b, y, x);
  y -= a / b * x;
  return g;
}
void inter(long long A, long long B, long long C, long long a, long long b,
           long long c, long long &x1, long long &x2) {
  while (a) {
    long long t = A / a;
    A -= t * a, B -= t * b, C -= t * c;
    swap(A, a);
    swap(B, b);
    swap(C, c);
  }
  if (c % b) puts("-1"), exit(0);
  x2 = -c / b;
  if ((C + B * x2) % A) puts("-1"), exit(0);
  x1 = (-C - B * x2) / A;
}
inline void UN(rec &a1, rec &b1, rec &a2, rec &b2) {
  num = 0;
  for (long long i = 0; i < a1.num; ++i) pri[num++] = a1.p[i].p;
  for (long long i = 0; i < a2.num; ++i) pri[num++] = a2.p[i].p;
  for (long long i = 0; i < b1.num; ++i) pri[num++] = b1.p[i].p;
  for (long long i = 0; i < b2.num; ++i) pri[num++] = b2.p[i].p;
  sort(pri, pri + num);
  num = unique(pri, pri + num) - pri;
  for (long long i = 0; i < num; ++i) A1[i] = a1.query(pri[i]);
  for (long long i = 0; i < num; ++i) A2[i] = a2.query(pri[i]);
  for (long long i = 0; i < num; ++i) B1[i] = b1.query(pri[i]);
  for (long long i = 0; i < num; ++i) B2[i] = b2.query(pri[i]);
  long long A = 0, B = 0, C = 0;
  long long flg1 = 0;
  long long x1, x2;
  for (long long i = 0; i < num; ++i) {
    long long a = B1[i].k, b = -B2[i].k, c = A1[i].k - A2[i].k;
    if (a == 0 && b == 0) {
      if (c) puts("-1"), exit(0);
      continue;
    }
    long long g = gcd(a, gcd(-b, mabs(c)));
    a /= g, b /= g, c /= g;
    if (!b) {
      if (c % a) puts("-1"), exit(0);
      if (-c / a < 0) puts("-1"), exit(0);
    }
    if (!A && !B) {
      A = a, B = b, C = c;
      continue;
    }
    if (!B) {
      if (b) {
        inter(A, B, C, a, b, c, x1, x2);
        flg1 = 1;
        break;
      }
      if (C / A != c / a) puts("-1"), exit(0);
      continue;
    }
    if (A * b == a * B) {
      if (c * A == C * a) continue;
      puts("-1"), exit(0);
    }
    inter(A, B, C, a, b, c, x1, x2), flg1 = 1;
    break;
  }
  if (flg1) {
    for (long long i = 0; i < num; ++i) {
      long long a = B1[i].k, b = -B2[i].k, c = A1[i].k - A2[i].k;
      if (a * x1 + b * x2 + c) puts("-1"), exit(0);
    }
    for (long long i = 0; i < num; ++i) {
      A1[i].k = A1[i].k + B1[i].k * x1;
      B1[i].k = 0;
    }
    for (long long i = 0; i < num; ++i) {
      a1.p[i] = A1[i], b1.p[i] = B1[i];
    }
    a1.num = b1.num = num;
    return;
  }
  long long g = exgcd(A, B, x1, x2);
  if (C % g) puts("-1"), exit(0);
  x1 *= -C / g, x2 *= -C / g;
  long long tx = mabs(-B / g), ty = mabs(A / g);
  if (C > 0 || !ty) {
    x1 = (x1 % tx + tx) % tx;
    if (B)
      x2 = -(A * x1 + C) / B;
    else
      x2 = 0;
  } else {
    x2 = (x2 % ty + ty) % ty;
    if (A)
      x1 = (-B * x2 - C) / A;
    else
      x1 = 0;
  }
  for (long long i = 0; i < num; ++i) {
    A1[i].k = A1[i].k + B1[i].k * x1;
    B1[i].k = tx * B1[i].k;
  }
  for (long long i = 0; i < num; ++i) {
    a1.p[i] = A1[i], b1.p[i] = B1[i];
  }
  a1.num = b1.num = num;
}
const long long P = 1e9 + 7;
inline long long ksm(long long x, long long y) {
  long long ans = 1;
  while (y) {
    if (y & 1) ans = ans * x % P;
    x = x * x % P;
    y >>= 1;
  }
  return ans;
}
signed main() {
  n = read();
  for (long long i = 1, a, b; i <= n; ++i) {
    a = read();
    b = read();
    A[i].init(a);
    B[i].init(b);
  }
  for (long long i = 2; i <= n; ++i) {
    UN(A[1], B[1], A[i], B[i]);
  }
  long long ans = 1;
  for (long long i = 0; i < A[1].num; ++i) {
    ans = ans * 1ll * ksm(A[1].p[i].p, A[1].p[i].k) % P;
  }
  printf("%lld\n", ans);
  return 0;
}
