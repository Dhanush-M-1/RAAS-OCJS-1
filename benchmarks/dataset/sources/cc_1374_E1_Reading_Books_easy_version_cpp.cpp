#include <bits/stdc++.h>
using namespace std;
template <typename T>
using minpq = priority_queue<T, vector<T>, greater<T>>;
int n, k, a, b;
long long t;
vector<long long> ve[2][2];
int main() {
  ios::sync_with_stdio(false);
  cin.tie(0);
  cin >> n >> k;
  for (int i = (0); i < (n); i++) {
    cin >> t >> a >> b;
    ve[a][b].push_back(t);
  }
  sort((ve[0][1]).begin(), (ve[0][1]).end());
  sort((ve[1][0]).begin(), (ve[1][0]).end());
  vector<long long> sums;
  for (long long x : ve[1][1]) sums.push_back(x);
  for (int i = 0; i < min(((int)(ve[0][1]).size()), ((int)(ve[1][0]).size()));
       i++) {
    sums.push_back(ve[0][1][i] + ve[1][0][i]);
  }
  sort((sums).begin(), (sums).end());
  if (((int)(sums).size()) < k) {
    cout << "-1\n";
    return 0;
  }
  cout << accumulate(sums.begin(), sums.begin() + k, 0LL) << '\n';
}
