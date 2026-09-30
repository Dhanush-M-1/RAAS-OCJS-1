#include <bits/stdc++.h>
using namespace std;
const int MAXN = 200005;
struct node {
  int smallerB;
  int smallerA;
  node() {
    smallerA = 0;
    smallerB = 0;
  }
};
node tree[4 * MAXN];
int items[MAXN];
int a, b;
void Modify(int v, int tl, int tr, int where) {
  if (tl == tr) {
    tree[v].smallerA = min(a, items[where]);
    tree[v].smallerB = min(b, items[where]);
  } else {
    int mid = (tl + tr) / 2;
    if (where <= mid) {
      Modify(2 * v, tl, mid, where);
    } else {
      Modify(2 * v + 1, mid + 1, tr, where);
    }
    tree[v].smallerA = tree[2 * v].smallerA + tree[2 * v + 1].smallerA;
    tree[v].smallerB = tree[2 * v].smallerB + tree[2 * v + 1].smallerB;
  }
}
int getSum(int v, int tl, int tr, int l, int r, int flag) {
  if (tl > r || l > tr) return 0;
  l = max(l, tl);
  r = min(r, tr);
  if (l == tl && r == tr) {
    if (flag == 0)
      return tree[v].smallerB;
    else
      return tree[v].smallerA;
  } else {
    int mid = (tl + tr) / 2;
    return getSum(2 * v, tl, mid, l, r, flag) +
           getSum(2 * v + 1, mid + 1, tr, l, r, flag);
  }
}
int main() {
  int N, K, Q;
  cin >> N >> K >> a >> b >> Q;
  for (int i = 1; i <= Q; i++) {
    int typ;
    cin >> typ;
    if (typ == 1) {
      int day, cnt;
      cin >> day >> cnt;
      items[day] += cnt;
      Modify(1, 1, N, day);
    } else {
      int p;
      cin >> p;
      int SumL = (p != 1 ? getSum(1, 1, N, 1, p - 1, 0) : 0);
      int SumR = (p != N - K + 1 ? getSum(1, 1, N, p + K, N, 1) : 0);
      cout << SumL + SumR << endl;
    }
  }
  return 0;
}
