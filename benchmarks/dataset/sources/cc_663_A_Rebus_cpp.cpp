#include <bits/stdc++.h>
using namespace std;
string s[505];
vector<int> P, N;
int main() {
  int id = 0, pos = 0, neg = 0;
  while (cin >> s[id]) {
    if (s[id] == "=") break;
    if (s[id] == "?") {
      if (id == 0 || s[id - 1] == "+")
        pos++;
      else
        neg++;
    }
    id++;
    cin >> s[id];
    if (s[id] == "=") break;
    id++;
  }
  int n;
  cin >> n;
  int dif = pos - neg;
  for (int i = 0; i < pos; i++) P.push_back(1);
  for (int i = 0; i < neg; i++) N.push_back(1);
  if (dif < n) {
    int i = 0;
    while (dif < n && i < pos) {
      if (dif + n - 1 < n) {
        dif += n - 1;
        P[i++] = n;
      } else {
        P[i++] = 1 + n - dif;
        dif = n;
      }
    }
    if (dif != n) {
      cout << "Impossible" << endl;
      return 0;
    }
  } else if (dif > n) {
    int i = 0;
    while (dif > n && i < neg) {
      if (dif - n + 1 > n) {
        dif -= n - 1;
        N[i++] = n;
      } else {
        N[i++] = 1 + dif - n;
        dif = n;
      }
    }
    if (dif != n) {
      cout << "Impossible" << endl;
      return 0;
    }
  }
  cout << "Possible" << endl;
  for (int i = 0, j = 0, k = 0; i <= id; i++) {
    if (s[i] == "?") {
      if (i == 0 || s[i - 1] == "+")
        cout << P[j++];
      else
        cout << N[k++];
    } else
      cout << " " << s[i];
  }
  cout << " " << n << endl;
  return 0;
}
