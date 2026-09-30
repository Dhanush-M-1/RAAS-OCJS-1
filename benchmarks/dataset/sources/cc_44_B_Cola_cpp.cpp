#include <bits/stdc++.h>
using namespace std;
int main() {
  double n;
  int a, b, c, ans = 0;
  cin >> n >> a >> b >> c;
  for (int i = min(int(n / 2), c); i >= 0; i--) {
    for (int j = min(int(n - 2 * i), b); j >= 0; j--) {
      if (a / 2 + j + i * 2 >= n) {
        ans++;
      }
    }
  }
  cout << ans;
  return 0;
}
