#include <bits/stdc++.h>
using namespace std;
void solve() {
  int n, k;
  cin >> n >> k;
  vector<int> mas(k, 0);
  for (int i = 0; i < n; i++) {
    int m;
    cin >> m;
    for (int i = 0; i < m; i++) {
      int x;
      cin >> x;
      mas[x - 1]++;
    }
  }
  for (int i = 0; i < k; i++) {
    if (mas[i] == 0) {
      cout << "NO";
      return;
    }
  }
  cout << "YES";
}
int main() {
  ios::sync_with_stdio(false);
  cin.tie(0);
  cout.tie(0);
  int tst = 1;
  while (tst--) solve();
  return 0;
}
