#include <bits/stdc++.h>
using namespace std;
int main(int argc, const char* argv[]) {
  int n;
  cin >> n;
  vector<int> v(n);
  for (int i = 0; i < n; ++i) {
    cin >> v[i];
  }
  vector<int> v2 = v;
  sort(v.rbegin(), v.rend());
  int m;
  cin >> m;
  for (int i = 0; i < m; ++i) {
    int k, ind;
    cin >> k >> ind;
    multiset<int> kmax;
    for (int i = 0; i < k; ++i) {
      kmax.insert(v[i]);
    }
    vector<int> seq;
    for (int i = 0; i < n; ++i) {
      auto it = kmax.find(v2[i]);
      if (it != kmax.end()) {
        kmax.erase(it);
        seq.push_back(v2[i]);
      }
      if (seq.size() == k) {
        break;
      }
    }
    cout << seq[ind - 1] << "\n";
  }
  return 0;
}
