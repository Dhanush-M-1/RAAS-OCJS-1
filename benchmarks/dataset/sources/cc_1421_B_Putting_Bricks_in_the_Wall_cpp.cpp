#include <bits/stdc++.h>
using namespace std;
const int inf = INT_MAX;
const long long int inf64 = LLONG_MAX;
vector<string> vect;
long long int n;
vector<pair<int, int> > d4{{-1, 0}, {1, 0}, {0, 1}, {0, -1}};
bool good(long long int xx, long long int yy) {
  return (xx < n and xx >= 0 and yy < n and yy >= 0);
}
int main() {
  ios::sync_with_stdio(false);
  cin.tie(0);
  long long int t;
  cin >> t;
  while (t--) {
    cin >> n;
    vect = vector<string>(n);
    long long int i;
    for (i = 0; i < n; i++) {
      cin >> vect[i];
    }
    vector<pair<int, int> > ans1, ans2;
    for (auto p : d4) {
      long long int xx, yy;
      xx = 0 + p.first;
      yy = 0 + p.second;
      if (good(xx, yy) and vect[xx][yy] == '1') ans1.push_back({xx, yy});
      xx = n - 1 + p.first;
      yy = n - 1 + p.second;
      if (good(xx, yy) and vect[xx][yy] == '0') ans1.push_back({xx, yy});
    }
    for (auto p : d4) {
      long long int xx, yy;
      xx = 0 + p.first;
      yy = 0 + p.second;
      if (good(xx, yy) and vect[xx][yy] == '0') ans2.push_back({xx, yy});
      xx = n - 1 + p.first;
      yy = n - 1 + p.second;
      if (good(xx, yy) and vect[xx][yy] == '1') ans2.push_back({xx, yy});
    }
    if (ans1.size() < ans2.size()) {
      cout << ans1.size() << '\n';
      for (auto x : ans1) cout << x.first + 1 << ' ' << x.second + 1 << '\n';
    } else {
      cout << ans2.size() << '\n';
      for (auto x : ans2) cout << x.first + 1 << ' ' << x.second + 1 << '\n';
    }
  }
  return 0;
}
