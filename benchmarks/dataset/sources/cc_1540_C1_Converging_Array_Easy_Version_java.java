/*
stream Butter!
eggyHide eggyVengeance
I need U
xiao rerun when
 */
import static java.lang.Math.max;
import static java.lang.Math.min;
import static java.lang.Math.abs;
import static java.lang.System.out;
import java.util.*;
import java.io.*;
import java.math.*;

public class Main
{
    static final int MAX = 10001;
    static final long MOD = 1000000007L;
    public static void main(String hi[]) throws Exception
    {
        BufferedReader infile = new BufferedReader(new InputStreamReader(System.in));
        StringTokenizer st = new StringTokenizer(infile.readLine());
        int N = Integer.parseInt(st.nextToken());
        int[] crr = readArr(N, infile, st);
        int[] brr = readArr(N-1, infile, st);
        int[] lolsum = new int[N-1];
        lolsum[0] = brr[0];
        for(int i=1; i < N-1; i++)
            lolsum[i] = lolsum[i-1]+brr[i];
        int[] prefixB = new int[N];
        for(int i=1; i < N; i++)
            prefixB[i] = prefixB[i-1]+lolsum[i-1];
        //System.out.println(prefixB[1]+" "+prefixB[2]+" "+prefixB[2]);
        int Q = Integer.parseInt(infile.readLine());
        StringBuilder sb = new StringBuilder();
        st = new StringTokenizer(infile.readLine());
        while(Q-->0)
        {
            int X = Integer.parseInt(st.nextToken());
            long[] dp = new long[MAX+1];
            for(int a=0; a <= crr[0]; a++)
                if(a >= X)
                    dp[a] = 1L;
            for(int t=1; t < N; t++)
            {
                long[] psums = new long[MAX+1];
                psums[0] = dp[0];
                for(int i=1; i <= MAX; i++)
                {
                    psums[i] = psums[i-1]+dp[i];
                    if(psums[i] >= MOD)
                        psums[i] -= MOD;
                }
                long[] next = new long[MAX+1];
                for(int prefix=max(0, (t+1)*X+prefixB[t]); prefix <= MAX; prefix++)
                {
                    int left = max(0, prefix-crr[t]);
                    long val = psums[prefix];
                    if(left > 0)
                    {
                        val -= psums[left-1];
                        if(val < 0)
                            val += MOD;
                    }
                    next[prefix] = val;
                }
                dp = next;
            }
            long res = 0L;
            for(long x: dp)
                res += x;
            res %= MOD;
            sb.append(res+"\n");
        }
        System.out.println(sb);
    }
    public static int[] readArr(int N, BufferedReader infile, StringTokenizer st) throws Exception
    {
        int[] arr = new int[N];
        st = new StringTokenizer(infile.readLine());
        for(int i=0; i < N; i++)
            arr[i] = Integer.parseInt(st.nextToken());
        return arr;
    }
}
/*
0-indexed
Basically, if arr[i+1]-arr[i] >= brr[i] (call this a gap), doing an operation here will do nothing
(one operation spreads the values apart to distance brr[i])
This means at the end, for all i, arr[i+1]-arr[i] >= brr[i]
This implies that the quantities cannot move across gaps
Even if the value changes to a point where an operation at i will do something, it will fix itself to satisfy the differences within each block

We only care about the first value of the final state, so anything that happens after the first gap doesn't affect F(arr, brr)
This means we can assume final[i] = arr[i]+brr[j] (where j < i for all j)
Notice final[i] = final[i-1]+brr[i-1] (for i > 1)
Factoring gives F(arr, brr) = final[1] = [(arr[1]+...+arr[k])+((brr[1])+(brr[1]+brr[2])+...+(brr[1]+...+brr[k]))]/k >= X
Count how many choices of arr and k exist such that (arr[1]+...+arr[k]) >= k*X-((brr[1])+(brr[1]+brr[2])+...+(brr[1]+...+brr[k]))
dp[i][sum of arr[1..i]]
Necessary, since otherwise it's not F(arr, brr)
Sufficient, since even if we already had a "gap" we can override the gap by adding a small enough value later on, which will break the condition anyways
 */