#include <bits/stdc++.h>
using namespace std;
int main() {
  int t, m, num = 1;
  string com;
  int arg;
  vector<pair<pair<int, int>, int> > d;
  cin >> t >> m;
  for (int _n((t)-1), q(0); q <= _n; q++) {
    sort((d).begin(), (d).end());
    cin >> com;
    if (com == "defragment") {
      if (d.size() > 0) {
        d[0].first.second -= d[0].first.first;
        d[0].first.first = 0;
        for (int _n((d.size() - 1)), i(1); i <= _n; i++) {
          int delta = d[i].first.first - d[i - 1].first.second;
          d[i].first.first -= delta;
          d[i].first.second -= delta;
        }
      }
      continue;
    }
    cin >> arg;
    if (com == "alloc") {
      int res = -1;
      if (d.size() > 0)
        if (d[0].first.first >= arg) {
          d.push_back(make_pair(make_pair(0, arg), num++));
          res = 1;
        }
      if (res == -1)
        for (int _n((d.size() - 1)), i(1); i <= _n; i++)
          if (d[i].first.first - d[i - 1].first.second >= arg) {
            d.push_back(make_pair(
                make_pair(d[i - 1].first.second, d[i - 1].first.second + arg),
                num++));
            res = 1;
            break;
          }
      if (res == -1)
        if (d.size() > 0)
          if (m - d[d.size() - 1].first.second >= arg) {
            d.push_back(make_pair(make_pair(d[d.size() - 1].first.second,
                                            d[d.size() - 1].first.second + arg),
                                  num++));
            res = 1;
          }
      if (res == -1)
        if (d.size() == 0)
          if (m >= arg) {
            d.push_back(make_pair(make_pair(0, arg), num++));
            res = 1;
          }
      if (res == -1)
        cout << "NULL" << endl;
      else
        cout << num - 1 << endl;
      continue;
    }
    if (com == "erase") {
      bool res = false;
      for (int i = 0; i < d.size(); i++)
        if (d[i].second == arg) {
          swap(d[i], d[d.size() - 1]);
          d.pop_back();
          res = true;
          break;
        }
      if (!res) cout << "ILLEGAL_ERASE_ARGUMENT" << endl;
      continue;
    }
  }
  return 0;
}
