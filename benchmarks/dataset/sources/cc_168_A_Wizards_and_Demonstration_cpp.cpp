#include <bits/stdc++.h>
using namespace std;
int main() {
  double n, x, y, p, q, r;
  int a;
  cin >> n >> x >> y;
  p = y / 100;
  q = n * p;
  r = q - x;
  a = r;
  if (r > a) {
    cout << a + 1;
  } else if (a < 0) {
    cout << "0";
  } else {
    cout << a;
  }
}
