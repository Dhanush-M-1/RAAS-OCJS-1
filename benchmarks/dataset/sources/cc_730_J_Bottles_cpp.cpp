#include <bits/stdc++.h>
using namespace std;
pair<int, int> dp[10005];
int a[105], b[105];
int main() {
  int n;
  scanf("%d", &n);
  int cap = 0, mx = 0;
  for (int i = 0; i < n; i++) {
    scanf("%d", &a[i]);
    cap += a[i];
  }
  for (int i = 0; i < n; i++) {
    scanf("%d", &b[i]);
    mx += b[i];
  }
  for (int i = 1; i <= mx; i++) dp[i] = make_pair(-1, -1);
  for (int i = 0; i < n; i++) {
    for (int j = mx; j >= 0; j--) {
      if (dp[j].first == -1) continue;
      int nxt = j + b[i];
      if (dp[nxt].first == -1)
        dp[nxt] = make_pair(dp[j].first + 1, dp[j].second + a[i]);
      else if (dp[nxt].first > dp[j].first + 1)
        dp[nxt] = make_pair(dp[j].first + 1, dp[j].second + a[i]);
      else if (dp[nxt].first == dp[j].first + 1)
        dp[nxt].second = max(dp[nxt].second, dp[j].second + a[i]);
    }
  }
  pair<int, int> mn = make_pair((int)1e9, 0);
  for (int i = cap; i <= mx; i++) {
    if (dp[i].first == -1) continue;
    if (dp[i].first < mn.first)
      mn = dp[i];
    else if (dp[i].first == mn.first)
      mn.second = max(mn.second, dp[i].second);
  }
  printf("%d %d\n", mn.first, cap - mn.second);
  return 0;
}
