#include <bits/stdc++.h>
using namespace std;
const int MAXN = 300000;
map<int, int> mapa;
map<pair<int, int>, vector<int>> pos;
vector<int> g[MAXN];
int ptr[MAXN];
int used[MAXN];
void euler(int v, vector<int> &res) {
  used[v] = true;
  for (; ptr[v] < (int)(g[v]).size();) {
    ++ptr[v];
    int u = g[v][ptr[v] - 1];
    euler(u, res);
    res.push_back(u);
  }
}
int main() {
  ios_base::sync_with_stdio(false);
  cin.tie(0);
  int n, s;
  cin >> n >> s;
  int k = 0;
  vector<int> a(n);
  for (int i = 0; i < n; ++i) {
    cin >> a[i];
  }
  vector<int> b = a;
  sort((b).begin(), (b).end());
  int m = 0;
  for (int i = 0; i < n; ++i) {
    if (a[i] == b[i]) {
      continue;
    }
    ++m;
    if (!mapa.count(b[i])) {
      mapa[b[i]] = k++;
    }
  }
  if (m > s) {
    cout << -1 << endl;
    return 0;
  }
  for (int i = 0; i < n; ++i) {
    if (a[i] == b[i]) {
      continue;
    }
    a[i] = mapa[a[i]];
    b[i] = mapa[b[i]];
    g[b[i]].push_back(a[i]);
    pos[{b[i], a[i]}].push_back(i);
  }
  vector<vector<int>> cycles;
  for (int i = 0; i < k; ++i) {
    if (!used[i]) {
      vector<int> arr;
      euler(i, arr);
      reverse((arr).begin(), (arr).end());
      cycles.push_back({});
      for (int i = 0; i < (int)(arr).size(); ++i) {
        int j = (i + 1) % (int)(arr).size();
        cycles.back().push_back(pos[{arr[i], arr[j]}].back());
        pos[{arr[i], arr[j]}].pop_back();
      }
    }
  }
  vector<vector<int>> res;
  if (s - m > 1 && (int)(cycles).size() > 1) {
    int len = min((int)(cycles).size(), s - m);
    res.push_back({});
    vector<int> newcycle;
    for (int i = (int)(cycles).size() - len; i < (int)(cycles).size(); ++i) {
      res.back().push_back(cycles[i].back());
      for (int j : cycles[i]) {
        newcycle.push_back(j);
      }
    }
    reverse((res.back()).begin(), (res.back()).end());
    for (int i = 0; i < len; ++i) {
      cycles.pop_back();
    }
    cycles.push_back(newcycle);
  }
  for (int i = 0; i < (int)(cycles).size(); ++i) {
    res.push_back(cycles[i]);
  }
  cout << (int)(res).size() << endl;
  for (int i = 0; i < (int)(res).size(); ++i) {
    cout << (int)(res[i]).size() << endl;
    for (int j : res[i]) {
      cout << j + 1 << " ";
    }
    cout << endl;
  }
}
