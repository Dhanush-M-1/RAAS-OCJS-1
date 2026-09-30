#include <bits/stdc++.h>
using namespace std;
const int mod = 1000 * 1000 * 1000 + 7;
const int INF = 1e9 + 100;
const long long LINF = 1e18 + 100;
int32_t main(void) {
  ios::sync_with_stdio(false);
  cin.tie(0);
  cout.tie(0);
  ;
  int a, b;
  cin >> a >> b;
  int x = 0, ans = 0;
  while (a) {
    ans += a;
    x += a;
    a = x / b;
    x %= b;
  }
  cout << ans << '\n';
  return 0;
}
