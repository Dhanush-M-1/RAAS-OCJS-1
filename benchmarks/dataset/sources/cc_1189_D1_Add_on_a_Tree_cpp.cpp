#include <bits/stdc++.h>
using namespace std;
void pr_init() {
  ios_base::sync_with_stdio(0);
  cin.tie(0);
  cout.tie(0);
}
vector<vector<long long> > tree;
vector<long long> deg;
void solve() {
  long long n;
  cin >> n;
  tree.assign(n + 1, vector<long long>());
  deg.assign(n + 1, 0);
  for (long long i = 0; i < n - 1; i++) {
    long long u, v;
    cin >> u >> v;
    deg[u]++;
    deg[v]++;
    tree[u].emplace_back(v);
    tree[v].emplace_back(u);
  }
  if (n == 2) {
    cout << "YES\n";
    return;
  } else if (n == 3) {
    cout << "NO\n";
    return;
  }
  bool is = true;
  for (long long i = 1; i <= n; i++) {
    if (deg[i] != 1 && deg[i] < 3) is = false;
  }
  if (is)
    cout << "YES\n";
  else
    cout << "NO\n";
}
int32_t main() {
  solve();
  return 0;
}
