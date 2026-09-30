#include <bits/stdc++.h>
using namespace std;
int main() {
  short l = 0;
  bool b = 0;
  vector<vector<short>> v(3);
  for (int i = 0; i < 3; i++)
    for (int j = 0; j < 2; j++) {
      cin >> l;
      v[i].push_back(l);
    }
  short x = (v[2][1] - v[0][1] + v[1][1]) / 2;
  short a[4];
  a[0] = v[0][0] - x;
  a[1] = x;
  a[2] = v[0][1] - v[1][1] + x;
  a[3] = v[1][1] - x;
  for (int i = 0; i < 4; i++)
    for (int j = i + 1; j < 4; j++) {
      if (a[i] == a[j]) {
        b = 1;
        j = 4;
        i = 4;
      }
    }
  for (int i = 0; i < 4; i++) {
    if (a[i] < 1 || a[i] > 9) {
      b = 1;
      i = 4;
    }
  }
  if (a[2] + a[0] != v[1][0]) b = 1;
  if (b)
    cout << -1 << endl;
  else {
    cout << a[0] << " " << a[1] << endl;
    cout << a[2] << " " << a[3] << endl;
  }
  return 0;
}
