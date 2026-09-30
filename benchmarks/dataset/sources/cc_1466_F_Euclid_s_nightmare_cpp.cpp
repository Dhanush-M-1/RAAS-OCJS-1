#include <bits/stdc++.h>
#define fst first
#define snd second
#define fore(i,a,b) for(int i=a,ThxDem=b;i<ThxDem;++i)
#define pb push_back
#define ALL(s) s.begin(),s.end()
#define FIN ios_base::sync_with_stdio(0);cin.tie(0);cout.tie(0)
#define SZ(s) int(s.size())
using namespace std;
typedef long long ll;
typedef pair<int,int> ii;

const int MAXN=5e5+10,MOD=1e9+7;

int add(int a, int b){a+=b;if(a>=MOD)a-=MOD;return a;}
int sub(int a, int b){a-=b;if(a<0)a+=MOD;return a;}
int mul(ll a, ll b){return a*b%MOD;}
int fpow(int a, ll b){
	int r=1;
	while(b){if(b&1)r=mul(r,a); b>>=1; a=mul(a,a);}
	return r;
}

int p[MAXN],am[MAXN],did[MAXN];
int find(int x){return p[x]=p[x]==x?x:find(p[x]);}
bool join(int x, int y){
	x=find(x); y=find(y);
	if(x==y)return 0;
	if(x!=y)p[x]=y,am[y]|=am[x];
	return 1;
}
int main(){FIN;
	int n,m,tot=0; cin>>n>>m;
	fore(i,0,m) p[i]=i;
	fore(i,0,n){
		int k; cin>>k;
		if(k==1){
			int x; cin>>x; x--;
			if(!am[find(x)]) am[find(x)]=1,did[i]=1,tot++;
		}
		if(k==2){
			int x,y; cin>>x>>y; x--; y--;
			int has=am[find(x)]&&am[find(y)];
			if(join(x,y)&&!has)did[i]=1,tot++;
		}
	}
	cout<<fpow(2,tot)<<" "<<tot<<"\n";
	fore(i,0,n)if(did[i])cout<<i+1<<" ";cout<<"\n";
}