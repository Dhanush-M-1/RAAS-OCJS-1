#include <bits/stdc++.h>
using namespace std;
void gi(int& x) {
  static char r[(1 << 17) + 16], *s = r, *l = r + (1 << 17);
  x = 0;
  if (!*s) memset(r, 0, (1 << 17)), cin.read(r, (1 << 17)), s = r;
  while (*s && (*s < 48 || *s > 57)) ++s;
  while (*s >= 48 && *s <= 57) {
    x = x * 10 + *s - 48, ++s;
    if (s == l) memset(r, 0, (1 << 17)), cin.read(r, (1 << 17)), s = r;
  }
  ++s;
}
char QQ[(1 << 17) + 16], *OP = QQ, *LP = QQ + (1 << 17);
void pn(int x) {
  char *B = OP, c, *E;
  int t;
  if (!x) t = x / 10, c = x - 10 * t + 48, *OP++ = c, x = t;
  while (x) t = x / 10, c = x - 10 * t + 48, *OP++ = c, x = t;
  E = OP - 1;
  while (B < E) swap(*B, *E), ++B, --E;
  if (OP > LP) cout.write(QQ, OP - QQ), OP = QQ;
}
void pc(char c) { *OP++ = c; }
int N, A[(1000006)], g, l, G[(1000006)], L[(1000006)], P[(1000006)], a, W;
long long S, X;
int main(void) {
  ios_base::sync_with_stdio(0);
  gi(N);
  for (int i(0); i < N; i++) gi(A[i + 1]);
  for (int k(1); k < N + 1; k++)
    if (A[k] > k)
      ++g, S += a = A[k] - k, --G[a], ++L[a], P[N - k] += 2 * A[k] - 1 - N,
                ++G[N - k], --L[N - k];
    else
      ++l, S += k - A[k], a = N - k + A[k], --G[a], ++L[a],
                          P[N - k] += 2 * A[k] - 1 - N, ++G[N - k], --L[N - k];
  X = S;
  for (int k(0); k < N; k++) {
    l += L[k], g += G[k], S += P[k] + l - g + 1;
    if (S < X) X = S, W = k + 1;
  }
  printf("%lld %d\n", X, W);
  return 0;
}
