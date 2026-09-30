#include <bits/stdc++.h>
using namespace std;
const int N = 1e6 + 10;
const int mod = 1e9 + 7;
int head[N];
int dis[N], ecnt;
int fa[N];
int cat[2005][2005];
long long gcd(long long a, long long b) {
  return a % b == 0 ? b : gcd(b, a % b);
}
long long qpow(long long base, long long n) {
  long long ans = 1;
  while (n) {
    if (n & 1) ans = (ans * base) % mod;
    n >>= 1;
    base = base * base % mod;
  }
  return ans;
}
struct Node {
  long long x, y;
} nd[N];
long long m, n, x, k, y;
struct EDGE {
  int u, v, nxt;
  long long w;
} e[N];
bool cmp(EDGE a, EDGE b) { return a.w < b.w; }
void add_edge(int u, int v, long long w) {
  e[ecnt].u = u;
  e[ecnt].v = v;
  e[ecnt].w = w;
  e[ecnt].nxt = head[u];
  head[u] = ecnt++;
}
int fd(int x) { return -1 == fa[x] ? x : fa[x] = fd(fa[x]); }
int c[N];
int lowbit(int x) { return x & (-x); }
void add(long long *c, int x, int y) {
  while (x <= n) {
    c[x] += y;
    x += lowbit(x);
  }
}
long long getsum(long long *c, int x) {
  long long res = 0;
  while (x > 0) {
    res += c[x];
    x -= lowbit(x);
  }
  return res;
}
int vis[N];
int p[N];
long long ans;
int C[55][55];
int dp[150][150];
long long posar[N], smar[N];
struct H {
  int p, s;
  friend bool operator<(H a, H b) { return a.p < b.p; }
} he[N];
bool cmp(int a, int b) { return a > b; }
priority_queue<int> q;
vector<int> v[30];
map<vector<int>, int> mp;
int ar[N];
int br[N];
vector<int> vv[10];
int main() {
  {
    cin >> n;
    for (int i = 1; i <= n; i++) {
      scanf("%d", &ar[i]);
      p[ar[i]] = i;
    }
    long long inv = 0;
    for (int i = 1; i <= n; i++) {
      inv += i - 1 - getsum(smar, p[i]);
      add(smar, p[i], 1);
      add(posar, p[i], p[i]);
      int l = 1;
      int r = n;
      int mid;
      while (l < r) {
        mid = 1 + l + r >> 1;
        if (getsum(smar, mid - 1) * 2 <= i)
          l = mid;
        else
          r = mid - 1;
      }
      mid = l;
      long long pre_cnt_sum = getsum(smar, mid),
                pre_pos_sum = getsum(posar, mid);
      long long mov =
          pre_cnt_sum * mid - pre_pos_sum - pre_cnt_sum * (pre_cnt_sum - 1) / 2;
      long long aft_cnt_sum = i - pre_cnt_sum;
      mov += getsum(posar, n) - pre_pos_sum - aft_cnt_sum * mid -
             aft_cnt_sum * (aft_cnt_sum + 1) / 2;
      cout << inv + mov << " \n"[i == n];
    }
  }
  return 0;
}
