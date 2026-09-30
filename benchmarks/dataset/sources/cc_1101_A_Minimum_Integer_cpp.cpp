#include <bits/stdc++.h>
using namespace std;
long long f[1000001];
long long pow(long long a, long long b, long long MOD) {
  long long x = 1, y = a;
  while (b > 0) {
    if (b % 2 == 1) {
      x = (x * y);
      if (x > MOD) x %= MOD;
    }
    y = (y * y);
    if (y > MOD) y %= MOD;
    b /= 2;
  }
  return x;
}
long long InverseEuler(long long n, long long MOD) {
  return pow(n, MOD - 2, MOD);
}
long long C(long long n, long long r, long long MOD) {
  return (f[n] *
          ((InverseEuler(f[r], MOD) * InverseEuler(f[n - r], MOD)) % MOD)) %
         MOD;
}
int main() {
  int q;
  cin >> q;
  while (q--) {
    long long l, r, d;
    cin >> l >> r >> d;
    long long ans1 = d;
    long long y = r / d;
    long long ans2 = (y + 1) * d;
    if (d >= l)
      cout << ans2 << endl;
    else
      cout << ans1 << endl;
  }
}
