#include <bits/stdc++.h>
using namespace std;
#define rep(i, a) for (int i = 0; i < (int)(a); i++)
#define sz(x) (int)(x).size()
#define pcnt __builtin_popcountll
typedef long long ll;
template<typename T>istream& operator>>(istream&i,vector<T>&v){rep(j,sz(v))i>>v[j];return i;}
template<typename T>string join(const vector<T>&v){stringstream s;rep(i,sz(v))s<<' '<<v[i];return s.str().substr(1);}
template<typename T>ostream& operator<<(ostream&o,const vector<T>&v){if(sz(v))o<<join(v);return o;}
template<typename T1,typename T2>istream& operator>>(istream&i,pair<T1,T2>&v){return i>>v.first>>v.second;}
template<typename T1,typename T2>ostream& operator<<(ostream&o,const pair<T1,T2>&v){return o<<v.first<<","<<v.second;}
template<typename T>bool mins(T& x,const T&y){if(x>y){x=y;return true;}else return false;}
template<typename T>bool maxs(T& x,const T&y){if(x<y){x=y;return true;}else return false;}
template<typename T>ll suma(const vector<T>&a){ll res(0);for(auto&&x:a)res+=x;return res;}

#ifdef _DEBUG
inline void dump() { cerr << endl; }
template <typename Head> void dump(Head &&head) { cerr << head; dump(); }
template <typename Head, typename... Tail> void dump(Head &&head, Tail &&... tail) { cerr << head << ", "; dump(forward<Tail>(tail)...); }
#define debug(...) do { cerr << __LINE__ << ":\t" << #__VA_ARGS__ << " = "; dump(__VA_ARGS__); } while (false)
#else
#define dump(...)
#define debug(...)
#endif

template <typename T> struct edge {
  int src, to;
  T cost;
  edge(int to, T cost) : src(-1), to(to), cost(cost) {}
  edge(int src, int to, T cost) : src(src), to(to), cost(cost) {}
  edge &operator=(const int &x) {
    to = x;
    return *this;
  }
  operator int() const { return to; }
};
template <typename T> using Edges = vector<edge<T>>;
template <typename T> using WeightedGraph = vector<Edges<T>>;
using UnWeightedGraph = vector<vector<int>>;
template <typename T> using Matrix = vector<vector<T>>;

const ll LINF = 1LL << 60;
const int INF = 1001001001;

/////////////////////////////////////////////////////////////////////

template<int mod>
struct ModInt {
  ll x;
  ModInt(ll x=0):x((x%mod+mod)%mod){}
  ModInt operator-() const { return ModInt(-x); }
  ModInt& operator+=(const ModInt a) {
    if ((x += a.x) >= mod) x -= mod;
    return *this;
  }
  ModInt& operator-=(const ModInt a) {
    if ((x += mod-a.x) >= mod) x -= mod;
    return *this;
  }
  ModInt& operator*=(const ModInt a) { (x *= a.x) %= mod; return *this; }
  ModInt operator+(const ModInt a) const { return ModInt(*this)+=a; }
  ModInt operator-(const ModInt a) const { return ModInt(*this)-=a; }
  ModInt operator*(const ModInt a) const { return ModInt(*this)*=a; }
  ModInt pow(ll t) const {
    if (!t) return 1;
    ModInt a = *this, r = 1;
    while (t) {
      if (t & 1) r *= a;
      a *= a;
      t >>= 1;
    }
    return r;
  }
  ModInt inv() const {
    ll a = x;
    ll b = mod;
    ll c = 1, d = 0;
    while (b) {
      ll t = a/b;
      a -= t*b; swap(a, b);
      c -= t*d; swap(c, d);
    }
    c %= mod;
    if (c < 0) c += mod;
    return c;
  }
  ModInt& operator/=(const ModInt a) { return (*this) *= a.inv(); }
  ModInt operator/(const ModInt a) const { return ModInt(*this)/=a; }
  bool operator==(const ModInt a) const { return x == a.x; }
  bool operator!=(const ModInt a) const { return x != a.x; }
  friend ostream &operator<<(ostream &os, const ModInt &p) { return os << p.x; }
  friend istream &operator>>(istream &is, ModInt &a) {
    ll t;
    is >> t;
    a = ModInt(t);
    return (is);
  }
};
const ll mod = 1000000007;
using mint = ModInt<mod>;

vector<mint> res(200030);

void solve()
{
  int n,m;
  scanf("%d %d", &n, &m);
  ll ans = 0;
  while (n>0) {
    int t = n%10;
    ans += res[m+t].x;
    n /= 10;
  }

  std::printf("%lld\n", ans%mod);
}

int main()
{
  vector<vector<mint>> dp(200030, vector<mint>(10));
  dp[0][0] = 1;
  rep(j, 200011) {
    rep(k, 9) dp[j+1][k+1] += dp[j][k];
    dp[j+1][0] += dp[j][9];
    dp[j+1][1] += dp[j][9];
    rep(k, 10) {
      res[j+1] += dp[j+1][k];
    }
  }

  int t; cin>>t;
  while(t--) solve();

  return 0;
}
