#include <bits/stdc++.h>
using namespace std;
const int maxn = 3e5 + 10;
int n, k;
char s[maxn];
int pre[maxn << 1], sz[maxn << 1];
vector<int> op[maxn];
int find(int x) { return x == pre[x] ? x : pre[x] = find(pre[x]); }
void merge(int x, int y) {
  int fx = find(x), fy = find(y);
  if (fy == 0) swap(fx, fy);
  if (fx != fy) pre[fy] = fx, sz[fx] += sz[fy];
}
int cal(int x) {
  int y = (x <= k) ? x + k : x - k;
  int fx = find(x), fy = find(y);
  if (fx == 0 || fy == 0) return sz[fx + fy];
  return min(sz[fx], sz[fy]);
}
int main() {
  scanf("%d%d", &n, &k);
  scanf("%s", s + 1);
  for (int i = 1; i <= k; i++) pre[i] = i, pre[i + k] = i + k, sz[i + k] = 1;
  for (int i = 1, c, x; i <= k; i++) {
    scanf("%d", &c);
    while (c--) {
      scanf("%d", &x);
      op[x].push_back(i);
    }
  }
  int res = 0;
  for (int i = 1; i <= n; i++) {
    if (op[i].size() == 1) {
      int x = op[i][0];
      res -= cal(x);
      if (s[i] == '1')
        pre[find(x + k)] = 0;
      else
        pre[find(x)] = 0;
      res += cal(x);
    } else if (op[i].size() == 2) {
      int x = op[i][0], y = op[i][1];
      if (s[i] == '1') {
        if (find(x) != find(y)) {
          res -= cal(x) + cal(y);
          merge(x, y), merge(x + k, y + k);
          res += cal(x);
        }
      } else {
        if (find(x + k) != find(y)) {
          res -= cal(x) + cal(y);
          merge(x + k, y), merge(x, y + k);
          res += cal(x);
        }
      }
    }
    printf("%d\n", res);
  }
  return 0;
}
