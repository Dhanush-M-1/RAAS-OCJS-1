import java.util.*;
import java.io.*;
import java.math.*;
public class Main{
    public static long[] bit;
    public static int n;
    public static long ver(int c,long sum){
        long r =(long) c*(long)(c-1);
        r/=(long)2;
        return sum-r;
    }
    public static void update(int x,long val){
        for(;x<=n;x+=x&-x) bit[x] += val;
    }
    public static long query(int x){
        long sum=0;
        for(;x>0;x-=x&-x) sum += bit[x];
        return sum;
    }
    public static void main(String[] args)throws IOException{
        BufferedReader br =  new BufferedReader(new InputStreamReader(System.in));
        PrintWriter out = new PrintWriter(System.out);
        StringTokenizer st = new StringTokenizer(br.readLine());
        n = Integer.parseInt(st.nextToken());
        long[] s = new long[n+1];
        st = new StringTokenizer(br.readLine());
        for(int i=1;i<=n;i++) s[i] = Long.parseLong(st.nextToken());
        int[] p = new int[n+1];
        bit = new long[n+1];
        for(int i=n;i>0;i--){
            int l = 1;
            int r = n+1;
            while(l<r){
                if(l==r-1) break;
                int c = l+r;
                c /= 2;
                if(ver(c,s[i]+query(c-1))<0){//sum from 1 to < c
                    r = c;
                }else l = c;
            }
            p[i] = l;
            update(p[i],(long)p[i]);
        }
        out.print(p[1]);
        for(int i=2;i<=n;i++) out.print(" "+p[i]);
        out.println("");
        out.flush();
   }
}