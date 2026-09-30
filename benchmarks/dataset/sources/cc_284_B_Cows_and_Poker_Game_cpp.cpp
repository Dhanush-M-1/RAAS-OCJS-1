#include <bits/stdc++.h>
using namespace std;
template <class T>
void pp(T v) {
  for (__typeof((v).begin()) it = (v).begin(); it != (v).end(); ++it)
    cout << *it << ' ';
  cout << endl;
}
template <class T>
void pp(T v, int n) {
  for (int i = 0; i < (int)n; i++) cout << v[i] << ' ';
  cout << endl;
}
template <class T>
inline void chmax(T &a, const T b) {
  a = max(a, b);
}
template <class T>
inline void chmin(T &a, const T b) {
  a = min(a, b);
}
const int INF = 1 << 28;
const double EPS = 1.0e-9;
static const int dx[] = {1, 0, -1, 0}, dy[] = {0, -1, 0, 1};
int main(void) {
  int N;
  cin >> N;
  string line;
  cin >> line;
  int A = 0, F = 0, I = 0;
  for (int i = 0; i < (int)N; i++) {
    switch (line[i]) {
      case 'A':
        A++;
        break;
      case 'F':
        F++;
        break;
      case 'I':
        I++;
        break;
    }
  }
  int ans = 0;
  for (int i = 0; i < (int)N; i++) {
    if (line[i] == 'A' || line[i] == 'I') {
      int irem = I - (line[i] == 'I' ? 1 : 0);
      if (irem <= 0) ans++;
    }
  }
  cout << ans << endl;
  return 0;
}
