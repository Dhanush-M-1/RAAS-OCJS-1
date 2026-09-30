#include <bits/stdc++.h>
using namespace std;
int n, k, tot, x, ans, a[1000000], b[1000000], f[1000000], ed[1000000],
    nt[1000000], nm[1000000], ss[1000000];
map<int, int> mp, rw;
bool v[1000000];
int gf(int x) {
  if (f[x] == x) return x;
  f[x] = gf(f[x]);
  return f[x];
}
int main() {
  scanf("%d %d", &n, &k);
  for (int i = (1); i <= (n); i++) {
    scanf("%d", &a[i]);
    b[i] = a[i];
  }
  sort(b + 1, b + n + 1);
  for (int i = (1); i <= (n); i++)
    if (b[i] != b[i - 1]) mp[b[i]] = i;
  for (int i = (1); i <= (n); i++)
    if (a[i] != b[i])
      k--;
    else
      v[i] = 1;
  for (int i = (1); i <= (n); i++)
    for (; v[mp[b[i]]]; mp[b[i]]++) {
      if (b[mp[b[i]]] != b[i]) {
        mp[b[i]] = 0;
        break;
      }
    }
  if (k < 0) {
    printf("-1");
    return 0;
  }
  for (int i = (1); i <= (n); i++)
    if (!v[i]) {
      tot++;
      for (x = i; x; x = mp[a[x]]) {
        for (mp[b[x]]++; v[mp[b[x]]]; mp[b[x]]++) {
          if (b[mp[b[x]]] != b[x]) {
            mp[b[x]] = 0;
            break;
          }
        }
        if (b[mp[b[x]]] != b[x]) mp[b[x]] = 0;
        v[x] = 1;
        if (ed[tot]) {
          nt[ed[tot]] = x;
          f[x] = f[ed[tot]];
        } else
          f[x] = x;
        ed[tot] = x;
        nm[f[x]]++;
      }
      nt[ed[tot]] = f[ed[tot]];
    }
  for (int i = (1); i <= (n); i++)
    if (a[i] != b[i]) {
      if (rw[a[i]]) {
        if (gf(i) != gf(rw[a[i]])) {
          nm[f[rw[a[i]]]] += nm[f[i]];
          f[f[i]] = f[rw[a[i]]];
          swap(nt[i], nt[rw[a[i]]]);
        }
      } else
        rw[a[i]] = i;
    }
  tot = 0;
  for (int i = (1); i <= (n); i++)
    if ((a[i] != b[i]) && (gf(i) == i)) ss[++tot] = i;
  if ((tot > 2) && (k > 2)) {
    ans = tot - ((tot) < (k) ? (tot) : (k)) + 1;
    x = nt[ss[ans]];
    for (int i = (ans + 1); i <= (tot); i++) {
      nt[ss[i - 1]] = nt[ss[i]];
      nm[ss[ans]] += nm[ss[i]];
    }
    nt[ss[tot]] = x;
  } else
    ans = tot;
  if (ans < tot)
    printf("%d\n", ans + 1);
  else
    printf("%d\n", ans);
  for (int i = (1); i <= (ans); i++) {
    printf("%d\n", nm[ss[i]]);
    for (x = ss[i]; nt[x] != ss[i]; x = nt[x]) printf("%d ", x);
    printf("%d\n", x);
  }
  if (ans < tot) {
    printf("%d\n", tot - ans + 1);
    for (int i = (tot); i >= (ans); i--) printf("%d ", nt[ss[i]]);
  }
}
