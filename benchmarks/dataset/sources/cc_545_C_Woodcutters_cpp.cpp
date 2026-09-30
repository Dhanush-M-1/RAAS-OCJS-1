#include <bits/stdc++.h>
using namespace std;
const int MOD = 1e9 + 7;
const int INF = INT_MAX;
const long long int LINF = LLONG_MAX;
int main() {
  ios_base::sync_with_stdio(false), cin.tie(NULL), cout.tie(NULL);
  int n, maxx = -INF, x, h, res = 0;
  cin >> n;
  pair<int, int> tree[n];
  for (int i = 0; i < n; ++i) cin >> tree[i].first >> tree[i].second;
  for (int i = 0; i < n; ++i) {
    int cx = tree[i].first, ch = tree[i].second;
    if (cx - ch > maxx) {
      res++;
      maxx = cx;
    } else if (i == n - 1 || cx + ch < tree[i + 1].first) {
      res++;
      maxx = cx + ch;
    } else {
      maxx = cx;
    }
  }
  cout << res;
  return 0;
}
