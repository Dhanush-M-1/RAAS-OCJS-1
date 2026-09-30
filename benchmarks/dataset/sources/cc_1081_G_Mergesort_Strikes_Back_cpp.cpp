#include <bits/stdc++.h>
const int MAXN = 1e5 + 20;
int n, k, M;
int inv[MAXN], pre_inv[MAXN];
void math_pre() {
  inv[1] = 1;
  for (int i = 2; i <= ((n < 4) ? 4 : n); ++i)
    inv[i] = 1ll * (M - M / i) * inv[M % i] % M;
  for (int i = 1; i <= n; ++i) pre_inv[i] = (pre_inv[i - 1] + inv[i]) % M;
}
struct map {
  static const int MAXMap = 2;
  int tot;
  struct pad {
    int key, val;
    pad() {}
    pad(const int &KEY, const int &VAL) : key(KEY), val(VAL) {}
  } node[MAXMap + 1];
  map() { tot = 0; }
  pad *find(const int &key) {
    pad *ret = node;
    while (ret - node < tot && ret->key != key) ++ret;
    return ret;
  }
  void insert(const pad &new_element) { node[tot++] = new_element; }
  pad *begin() { return &node[0]; }
  pad *end() { return &node[tot]; }
} Map;
void solve(const int &l, const int &r, const int &h) {
  if (l >= r || h <= 1) {
    int len = r - l + 1;
    map::pad *it = Map.find(len);
    if (it == Map.end())
      Map.insert(map::pad(len, 1));
    else
      ++it->val;
    return;
  }
  int mid = (l + r) >> 1;
  solve(l, mid, h - 1), solve(mid + 1, r, h - 1);
}
int calc(const int &len1, const int &len2) {
  int ret = 0;
  for (int i = 1; i <= len1; ++i)
    ret = ((ret + 1ll * inv[2] * len2 % M -
            (pre_inv[i + len2] - pre_inv[i + 1 - 1])) %
               M +
           M) %
          M;
  return ret;
}
int main() {
  scanf("%d%d%d", &n, &k, &M);
  math_pre();
  solve(1, n, k);
  int ans = 0;
  for (map::pad *it = Map.begin(); it != Map.end(); ++it) {
    int len = it->key, cnt = it->val;
    ans = (ans + 1ll * cnt * len % M * (len - 1) % M * inv[4] % M) % M;
  }
  for (map::pad *it1 = Map.begin(); it1 != Map.end(); ++it1)
    for (map::pad *it2 = Map.begin(); it2 != Map.end(); ++it2) {
      if (it1 == it2) {
        int len = it1->key, cnt = 1ll * (0 + (it1->val - 1)) * it1->val / 2 % M;
        ans = (ans + 1ll * cnt * calc(len, len) % M) % M;
      } else if (it1->key < it2->key) {
        int len1 = it1->key, len2 = it2->key,
            cnt = 1ll * it1->val * it2->val % M;
        ans = (ans + 1ll * cnt * calc(len1, len2) % M) % M;
      }
    }
  printf("%d", ans);
}
