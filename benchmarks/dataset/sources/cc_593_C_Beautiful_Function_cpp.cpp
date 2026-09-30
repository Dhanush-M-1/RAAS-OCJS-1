#include <bits/stdc++.h>
using namespace std;
int N;
vector<int> xs, ys;
string generateFunc(int i, int x) {
  char str[1024];
  sprintf(str, "(%d*((1-abs((t-%d)))+abs((abs((t-%d))-1))))", x / 2, i, i);
  return string(str);
}
string solve(vector<int>& xs) {
  string rv;
  rv += generateFunc(0, xs[0]);
  for (int i = 1; i < N; ++i)
    rv = "(" + rv + "+" + generateFunc(i, xs[i]) + ")";
  return rv;
}
int main() {
  cin >> N;
  for (int i = 0; i < N; ++i) {
    int x, y, r;
    cin >> x >> y >> r;
    xs.emplace_back(x);
    ys.emplace_back(y);
  }
  cout << solve(xs) << endl;
  cout << solve(ys) << endl;
  return 0;
}
