#include <bits/stdc++.h>
using namespace std;
int main() {
  ios::sync_with_stdio(false);
  cin.tie(0);
  int tt;
  cin >> tt;
  while (tt--) {
    int n;
    cin >> n;
    vector<int> a(n);
    for (int i = 0; i < n; ++i) {
      cin >> a[i];
    }
    if (a[0] + a[1] <= a.back()) {
      cout << "1 2 " << n << '\n';
      goto nx;
    }
    cout << "-1\n";
  nx:;
  }
  return 0;
}
