#include <bits/stdc++.h>
using namespace std;
int main() {
  int n, a[3], q;
  cin >> n;
  for (int i = 0; i < 3; i++) a[i] = 0;
  for (int i = 0; i < 3; i++) {
    for (int j = 0; j < n - i; j++) {
      cin >> q;
      a[i] += q;
    }
  }
  cout << a[0] - a[1] << endl;
  cout << a[1] - a[2];
  return 0;
}
