#include <bits/stdc++.h>
using namespace std;
const long long maxN = 2 * 100224;
struct BIT {
  long long data[maxN] = {0};
  void update(long long idx, long long val) {
    while (idx < maxN) {
      data[idx] += val;
      idx += idx & -idx;
    }
  }
  void update(long long l, long long r, long long val) {
    update(l, val);
    update(r + 1, -val);
  }
  long long query(long long idx) {
    long long res = 0;
    while (idx > 0) {
      res += data[idx];
      idx -= idx & -idx;
    }
    return res;
  }
  long long query(long long l, long long r) { return query(r) - query(l); }
};
struct LazyBIT {
  BIT bitAdd, bitSub;
  void update(long long l, long long r, long long val) {
    bitAdd.update(l, r, val);
    bitSub.update(l, r, (l - 1) * val);
    bitSub.update(r + 1, (-r + l - 1) * val);
  }
  long long query(long long idx) {
    return idx * bitAdd.query(idx) - bitSub.query(idx);
  }
  long long query(long long l, long long r) { return query(r) - query(l - 1); }
};
long long parent[maxN];
long long rnk[maxN];
long long lfmost[maxN];
long long rtmost[maxN];
long long vis[maxN];
void make_set(long long v) {
  parent[v] = v;
  rnk[v] = 0;
  lfmost[v] = v;
  rtmost[v] = v;
}
long long find_set(long long v) {
  if (v == parent[v]) return v;
  return parent[v] = find_set(parent[v]);
}
void union_sets(long long a, long long b) {
  a = find_set(a);
  b = find_set(b);
  if (a != b) {
    if (rnk[a] < rnk[b]) swap(a, b);
    parent[b] = a;
    lfmost[a] = min(lfmost[a], lfmost[b]);
    rtmost[a] = max(rtmost[a], rtmost[b]);
    if (rnk[a] == rnk[b]) rnk[a]++;
  }
}
int main() {
  ios_base::sync_with_stdio(0);
  cin.tie(NULL);
  long long n;
  cin >> n;
  vector<long long> v(n);
  for (int i = 0; i < n; i++) {
    cin >> v[i];
  }
  long long sum = 0;
  LazyBIT B;
  for (long long i = 1; i <= n; i++) {
    B.update(i, i, sum);
    sum += i;
  }
  vector<long long> haha;
  for (int i = n - 1; i >= 0; i--) {
    long long lo = 1;
    long long hi = n;
    long long mid;
    long long ans;
    long long val;
    long long temp;
    while (lo <= hi) {
      mid = (lo + hi) / 2;
      if (vis[mid] == 1) {
        if (lfmost[find_set(mid)] - 1 >= lo)
          mid = lfmost[find_set(mid)] - 1;
        else if (rtmost[find_set(mid)] + 1 <= hi)
          mid = rtmost[find_set(mid)] + 1;
        else
          break;
      }
      val = B.query(mid, mid);
      if (val == v[i]) {
        ans = mid;
        hi = mid - 1;
      } else if (v[i] > val) {
        lo = mid + 1;
      } else if (v[i] < val) {
        hi = mid - 1;
      }
    }
    B.update(ans, n, -ans);
    vis[ans] = 1;
    make_set(ans);
    if (vis[ans] == 1 && vis[ans + 1] == 1) union_sets(ans, ans + 1);
    if (vis[ans] == 1 && vis[ans - 1] == 1) union_sets(ans, ans - 1);
    haha.push_back(ans);
  }
  reverse(haha.begin(), haha.end());
  for (auto u : haha) {
    cout << u << " ";
  }
}
