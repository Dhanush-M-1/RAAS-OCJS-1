#include<bits/stdc++.h>
#define pii pair<int,int>
#define fi first
#define sc second
#define pb push_back
#define ll long long
#define trav(v,x) for(auto v:x)
#define all(x) (x).begin(), (x).end()
#define VI vector<int>
#define VLL vector<ll>
using namespace std;
const int N = 1e6 + 100;
const ll mod = 1e9 + 7;

signed main()
{
	ios::sync_with_stdio(0);
	cin.tie(0);
    int n;
    cin >> n;
    VI mx(n + 1);
    for(int i = 1; i <= n; i++)
        cin >> mx[i];
    VI b(n + 1);
    for(int i = 1; i < n; i++)
        cin >> b[i], b[i] += b[i - 1];
    for(int i = 1; i < n; i++)
        b[i] += b[i - 1];
    vector<VLL> dp(n + 1, VLL(10010, 0));
//    VLL ans(300);
    ll ans = 0;
    int x;
    cin >> x;
    cin >> x;
  //  for(int x = -101; x <= 101; x++)
    {
        dp[0][0] = 1;
        int suma = 0;
        for(int i = 1; i <= n; i++)
        {
            fill(all(dp[i]), 0);
            for(int val = 0; val <= mx[i]; val++)
            {
                int mn = b[i - 1] + x * i - val;
                mn = max(mn, 0);
                for(int bf = mn; bf <= suma; bf++)
                {
                    dp[i][val + bf] += dp[i - 1][bf];
                    if(dp[i][val + bf] >= mod)dp[i][val + bf] -= mod;
                }
            }
            suma += mx[i];
        }
//        ans[x + 110] = 0;
//        for(int i = 0; i <= suma; i++)
//        {
//            ans[x + 110] += dp[n][i];
//            if(ans[x + 110] >= mod)ans[x + 110] -= mod;
//        }
            ans = 0;
            for(int i = 0; i <= suma; i++)
        {
            ans+= dp[n][i];
            if(ans >= mod)ans -= mod;
        }
        cout << ans << '\n';
    }
//    int tt;
//    cin >> tt;
//    while(tt --)
//    {
//        int x;
//        cin >> x;
//        x = max(x, -101);
//        x = min(x, 101);
//        cout << ans[x + 110] << '\n';
//    }
}
