#include <bits/stdc++.h>
using namespace std;
const int modulo = 1000000007;
int N, E[200010], lvl[200010], first[100010], M[200010][18], max2[200010],
    S1[100010], S2[100010], nr[100010], droot[100010], indd[100010], T[100010],
    finalsol1[100010], finalsol2[100010];
vector<int> V[100010], C[100010], pd1[100010], pd2[100010];
int add(int x, int y) {
  x += y;
  return x >= modulo ? x - modulo : x;
}
int mult(int x, int y) {
  int ret = ((long long)x * y) % modulo;
  return ret < 0 ? ret + modulo : ret;
}
int sub(int x, int y) {
  x -= y;
  return x < 0 ? x + modulo : x;
}
void euler(int x, int& poz, int l = 0, int t = -1) {
  T[x] = t;
  for (auto i : V[x])
    if (t != i) {
      E[poz] = x;
      lvl[poz++] = l;
      euler(i, poz, l + 1, x);
    }
  E[poz] = x;
  lvl[poz++] = l;
}
void precompute_lca() {
  int poz = 0;
  euler(0, poz);
  memset(first, -1, sizeof(first));
  for (int i = 0; i < 2 * N - 1; i++)
    if (first[E[i]] == -1) first[E[i]] = i;
  for (int i = 0; i < 2 * N - 1; i++) M[i][0] = i;
  for (int k = 1; (1 << k) <= 2 * N - 1; k++)
    for (int i = 0; i + (1 << k) <= 2 * N - 1; i++) {
      int l = M[i][k - 1], r = M[i + (1 << (k - 1))][k - 1];
      M[i][k] = lvl[l] < lvl[r] ? l : r;
    }
  for (int k = 1; (1 << k) <= 2 * N - 1; k++) max2[1 << k] = 1;
  for (int i = 1; i <= 2 * N - 1; i++) max2[i] += max2[i - 1];
}
int lca(int x, int y) {
  int a = first[x], b = first[y];
  if (a > b) swap(a, b);
  int len = b - a + 1, k = max2[len];
  int l = M[a][k], r = M[b - (1 << k) + 1][k];
  if (lvl[l] < lvl[r])
    return E[l];
  else
    return E[r];
}
void calc_S(int x, int d = 0) {
  nr[x] = 1;
  droot[x] = d;
  for (int i = 0; i < V[x].size(); i++) {
    int y = V[x][i];
    if (y == T[x]) continue;
    calc_S(y, add(d, C[x][i]));
    S1[x] = add(S1[x], add(S1[y], mult(C[x][i], nr[y])));
    S2[x] = add(S2[x], add(S2[y], add(mult(nr[y], mult(C[x][i], C[x][i])),
                                      mult(2, mult(C[x][i], S1[y])))));
    nr[x] += nr[y];
  }
}
void calc_pd(int x, int ind = 0) {
  int up = 0;
  indd[x] = ind;
  if (T[x] != -1) {
    up = add(pd1[T[x]][ind], mult(C[T[x]][ind], N - nr[x]));
  }
  up = add(up, S1[x]);
  for (int i = 0; i < V[x].size(); i++)
    if (V[x][i] != T[x]) {
      pd1[x].push_back(sub(sub(up, S1[V[x][i]]), mult(C[x][i], nr[V[x][i]])));
    } else {
      pd1[x].push_back(-1);
    }
  up = 0;
  if (T[x] != -1) {
    up = add(
        add(pd2[T[x]][ind], mult(mult(N - nr[x], C[T[x]][ind]), C[T[x]][ind])),
        mult(mult(2, C[T[x]][ind]), pd1[T[x]][ind]));
  }
  up = add(up, S2[x]);
  for (int i = 0; i < V[x].size(); i++)
    if (V[x][i] != T[x]) {
      int y = V[x][i];
      pd2[x].push_back(
          sub(sub(sub(up, S2[y]), mult(mult(C[x][i], C[x][i]), nr[y])),
              mult(mult(2, C[x][i]), S1[y])));
    } else {
      pd2[x].push_back(-1);
    }
  for (int i = 0; i < V[x].size(); i++)
    if (V[x][i] != T[x]) calc_pd(V[x][i], i);
}
int calc_d(int x, int y) {
  int z = lca(x, y);
  return sub(add(droot[x], droot[y]), mult(2, droot[z]));
}
int main() {
  scanf("%d", &N);
  for (int i = 0; i < N - 1; i++) {
    int a, b, c;
    scanf("%d %d %d", &a, &b, &c);
    a--, b--;
    V[a].push_back(b);
    V[b].push_back(a);
    C[a].push_back(c);
    C[b].push_back(c);
  }
  precompute_lca();
  calc_S(0);
  calc_pd(0);
  int Q;
  scanf("%d", &Q);
  for (int i = 0; i < N; i++)
    if (T[i] != -1) {
      int t = T[i], ind = indd[i];
      finalsol1[i] = add(pd1[T[i]][indd[i]], mult(C[T[i]][indd[i]], N - nr[i]));
      finalsol2[i] =
          add(add(pd2[t][ind], mult(mult(C[t][ind], C[t][ind]), N - nr[i])),
              mult(mult(2, C[t][ind]), pd1[t][ind]));
    } else {
      finalsol1[i] = finalsol2[i] = 0;
    }
  for (int i = 0; i < Q; i++) {
    int u, v;
    scanf("%d %d", &u, &v);
    u--, v--;
    int ret = 0;
    int duv = calc_d(u, v);
    if (lca(u, v) == v) {
      ret = add(ret, mult(N - nr[v], mult(duv, duv)));
      ret = add(ret, finalsol2[v]);
      ret = add(ret, mult(mult(2, duv), finalsol1[v]));
      ret = mult(ret, -2);
      ret = add(ret, add(S2[u], finalsol2[u]));
    } else {
      ret = add(ret, mult(nr[v], mult(duv, duv)));
      ret = add(ret, S2[v]);
      ret = add(ret, mult(mult(2, duv), S1[v]));
      ret = mult(ret, 2);
      ret = sub(ret, add(S2[u], finalsol2[u]));
    }
    printf("%d\n", ret);
  }
  return 0;
}
