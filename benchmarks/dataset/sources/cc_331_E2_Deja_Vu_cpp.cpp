#include <bits/stdc++.h>
using namespace std;
const int MAX = 100 + 10;
const int Mod = (int)1e9 + 7;
int n, m;
int g[MAX][MAX], can[MAX][MAX];
vector<int> p[MAX][MAX];
int get(vector<int> &before, int kind, int could, int have) {
  int j;
  int now = 0, cc = have;
  for (; now < (int)before.size(); now++) {
    if (now + 1 == (int)before.size()) break;
    int a = before[now];
    int b = before[now + 1];
    if (g[a][b] != kind || (!could && can[a][b])) return 0;
    vector<int> &nL = p[a][b];
    for ((j) = (0); (j) != ((int)nL.size()); ++(j)) {
      if (cc < (int)before.size()) {
        if (nL[j] == before[cc])
          cc++;
        else
          return 0;
      } else {
        while (j < (int)nL.size()) {
          before.push_back(nL[j++]);
          if ((int)before.size() > 2 * n + 1) return 0;
        }
        cc = before.size();
        break;
      }
    }
  }
  return cc == (int)before.size();
}
int isCan(int a, int b) {
  int i;
  vector<int> &L = p[a][b];
  int len = L.size();
  for ((i) = (0); (i) <= (len - 2); ++(i))
    if (L[i] == a && L[i + 1] == b) return 1;
  return 0;
}
int f_Before[MAX][MAX][MAX], f_After[MAX][MAX][MAX], tmp[MAX][MAX][MAX];
int Before[MAX][MAX], After[MAX][MAX];
int ans[MAX];
void work(int u, int kind, int f[MAX][MAX][MAX]) {
  int v;
  for ((v) = (1); (v) <= (n); ++(v))
    if (g[u][v] == kind && !can[v][u]) {
      vector<int> after;
      after.push_back(u);
      after.push_back(v);
      if (!get(after, kind, kind != 2, 1)) continue;
      int Len = after.size();
      f[u][after[Len - 1]][Len - 1]++;
    }
}
void add(int &a, int b) {
  a += b;
  if (a >= Mod) a -= Mod;
}
void work2(int a, int b) {
  int i, j;
  vector<int> &L = p[a][b];
  int len = L.size();
  for ((i) = (0); (i) <= (len - 2); ++(i))
    if (L[i] == a && L[i + 1] == b) {
      vector<int> before, after;
      for (j = i; j >= 0; --j) before.push_back(L[j]);
      for (j = i + 1; j < len; ++j) after.push_back(L[j]);
      if (get(before, 2, 0, before.size()) && get(after, 1, 1, after.size())) {
        reverse(before.begin(), before.end());
        int A = before[0];
        int B = after[after.size() - 1];
        int Len = before.size() + after.size() - 1;
        int k, l, o;
        for ((l) = (1); (l) <= (n); ++(l))
          for ((o) = (0); (o) <= (2 * n); ++(o))
            if (tmp[A][l][o])
              for ((k) = (1); (k) <= (n); ++(k))
                if (g[k][l] == 1 && p[k][l].size() == 0 && Len + 1 + o <= 2 * n)
                  add(f_After[k][B][Len + 1 + o], tmp[A][l][o]);
      } else
        return;
    }
  return;
}
int tot = 0, First;
void Dp1(int f[MAX][MAX][MAX], int after[MAX][MAX]) {
  int i, j, k, l, o;
  memset(tmp, 0, sizeof tmp);
  for ((i) = (1); (i) <= (n); ++(i)) tmp[i][i][0] = 1;
  int up = 2 * n;
  for ((l) = (1); (l) <= (up); ++(l))
    for ((i) = (1); (i) <= (n); ++(i))
      for ((j) = (1); (j) <= (n); ++(j))
        for ((o) = (0); (o) <= (l); ++(o))
          if (f[i][j][o])
            for ((k) = (1); (k) <= (n); ++(k))
              if (tmp[j][k][l - o])
                add(tmp[i][k][l],
                    (long long)f[i][j][o] * tmp[j][k][l - o] % Mod);
  for ((i) = (1); (i) <= (n); ++(i))
    for ((j) = (1); (j) <= (n); ++(j))
      for ((l) = (0); (l) <= (2 * n); ++(l)) add(after[i][l], tmp[i][j][l]);
}
void Dp2(int f[MAX][MAX][MAX], int after[MAX][MAX]) {
  int i, j, k, l;
  for ((i) = (1); (i) <= (n); ++(i)) after[i][0] = 1;
  int up = 2 * n;
  for ((l) = (1); (l) <= (up); ++(l))
    for ((i) = (1); (i) <= (n); ++(i))
      for ((j) = (1); (j) <= (n); ++(j))
        for ((k) = (0); (k) != (l); ++(k))
          add(After[i][l], (long long)f[i][j][l - k] * After[j][k] % Mod);
}
void check(int a, int b) {
  int i, j;
  vector<int> &L = p[a][b];
  int len = L.size();
  for ((i) = (0); (i) <= (len - 2); ++(i))
    if (L[i] == a && L[i + 1] == b) {
      vector<int> before, after;
      for (j = i; j >= 0; --j) before.push_back(L[j]);
      for (j = i + 1; j < len; ++j) after.push_back(L[j]);
      if (get(before, 2, 0, before.size()) && get(after, 1, 1, after.size())) {
        reverse(before.begin(), before.end());
        int A = before[0];
        int B = after[after.size() - 1];
        int Len = before.size() + after.size() - 1;
        int l1, l2;
        for ((l1) = (0); (l1) <= (2 * n); ++(l1))
          for ((l2) = (0); (l2) <= (2 * n); ++(l2))
            if (l1 + l2 + Len <= 2 * n)
              add(ans[l1 + l2 + Len],
                  (long long)Before[A][l1] * After[B][l2] % Mod);
      } else
        return;
    }
  return;
}
int main() {
  int i, j;
  scanf("%d%d", &n, &m);
  for ((i) = (1); (i) <= (m); ++(i)) {
    int a, b;
    scanf("%d%d", &a, &b);
    if (!First) First = a;
    g[a][b] = 1;
    g[b][a] = 2;
    int k, first;
    scanf("%d", &k);
    while (k--) {
      scanf("%d", &first);
      p[a][b].push_back(first);
    }
    p[b][a] = p[a][b];
    reverse(p[b][a].begin(), p[b][a].end());
  }
  for ((i) = (1); (i) <= (n); ++(i))
    for ((j) = (1); (j) <= (n); ++(j))
      if (g[i][j] == 1 && isCan(i, j)) can[j][i] = can[i][j] = 1;
  for ((i) = (1); (i) <= (n); ++(i)) work(i, 1, f_After);
  for ((i) = (1); (i) <= (n); ++(i)) work(i, 2, f_Before);
  Dp1(f_Before, Before);
  for ((i) = (1); (i) <= (n); ++(i))
    for ((j) = (1); (j) <= (n); ++(j))
      if (g[i][j] == 1 && can[i][j] == 1) work2(i, j);
  Dp2(f_After, After);
  for ((i) = (1); (i) <= (n); ++(i))
    for ((j) = (1); (j) <= (n); ++(j))
      if (g[i][j] == 1 && can[i][j] == 1) check(i, j);
  for ((i) = (1); (i) <= (2 * n); ++(i)) cout << ans[i] << endl;
  return 0;
}
