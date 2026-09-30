
import java.io.*;
import java.util.*;
public class Main {
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////                                                                                                               /////////
////////                                                                                                               /////////
////////   HHHH        HHHH  EEEEEEEEEEEEE   MMMM          MMMM         OOOOOO             SSSSSSS      EEEEEEEEEEEEE  /////////
////////   HHHH        HHHH  EEEEEEEEEEEEE   MMMMMM      MMMMMM      OOO      OOO        SSSS   SSS     EEEEEEEEEEEEE  /////////
////////   HHHH        HHHH  EEEEE           MMMM MMM  MMM MMMM    OOO          OOO    SSSS       SSS   EEEEE          /////////
////////   HHHH        HHHH  EEEEE           MMMM  MMMMMM  MMMM   OOO            OOO   SSSS             EEEEE          /////////
////////   HHHH        HHHH  EEEEE           MMMM          MMMM  OOO              OOO   SSSSSSS         EEEEE          /////////
////////   HHHHHHHHHHHHHHHH  EEEEEEEEEEE     MMMM          MMMM  OOO              OOO      SSSSSS       EEEEEEEEEEE    /////////
////////   HHHHHHHHHHHHHHHH  EEEEEEEEEEE     MMMM          MMMM  OOO              OOO         SSSSSSS   EEEEEEEEEEE    /////////
////////   HHHH        HHHH  EEEEE           MMMM          MMMM   OOO            OOO              SSSS  EEEEE          /////////
////////   HHHH        HHHH  EEEEE           MMMM          MMMM    OOO          OOO     SSS       SSSS  EEEEE          /////////
////////   HHHH        HHHH  EEEEEEEEEEEEE   MMMM          MMMM      OOO      OOO        SSS    SSSS    EEEEEEEEEEEEE  /////////
////////   HHHH        HHHH  EEEEEEEEEEEEE   MMMM          MMMM         OOOOOO             SSSSSSS      EEEEEEEEEEEEE  /////////
////////                                                                                                               /////////
////////                                                                                                               /////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        public static void main(String[] args) throws IOException, InterruptedException {
        Scanner sc = new Scanner(System.in);
        PrintWriter pw = new PrintWriter(System.out);
        int n = sc.nextInt();
        int[] a = new int[n];
        int[] pos = new int[n+1];
        for (int i =0;i<n;i++){
            a[i]=sc.nextInt();
            pos[a[i]]=i+1;
        }
        FenwickTree fn = new FenwickTree(n);
        long[] ans = new long[n+1];
        //count inversions
        for (int i =n-1;i>=0;i--){
            ans[a[i]]+=fn.rsq(a[i]);
            fn.point_update(a[i],1);
        }
        for (int i =1;i<=n;i++)ans[i]+=ans[i-1];
//        pw.println(Arrays.toString(ans));
        //
        fn= new FenwickTree(n);
        FenwickTree fn2 = new FenwickTree(n);
        fn.point_update(pos[1],1);
        fn2.point_update(pos[1],pos[1]);
        for (int i =2;i<=n;i++){
            fn.point_update(pos[i],1);
            fn2.point_update(pos[i],pos[i]);
            int low = 1;
            int hi = n;
            int idx=0;
            while (low<=hi){
                int mid = low+hi >>1;
                if (fn.rsq(mid)>=(i+1)/2){
                    idx= mid;
                    hi = mid-1;
                }
                else low=mid+1;
            }

            long pre= +sumrange(idx-(i+1)/2+1,idx-1);
            pre-=fn2.rsq(idx-1);
            if (fn.rsq(idx)==1)pre=0;
            int add=i-(i+1)/2;
            long after=fn2.rsq(idx+1,n)-sumrange(idx+1,idx+add);
            if (add==0)after=0;
            ans[i]+=pre+after;
        }
        for (int i =1;i<=n;i++)
            pw.print(ans[i]+" ");
        pw.println();
        pw.flush();
    }
    static long sumrange(int a, int b){
            return -1l*a*(a-1)/2 + 1l*b*(b+1)/2;
    }

    static class FenwickTree { // one-based DS

        int n;
        long[] ft;

        FenwickTree(int size) { n = size; ft = new long[n+1]; }

        long rsq(int b) //O(log n)
        {
            long sum = 0;
            while(b > 0) { sum += ft[b]; b -= b & -b;}		//min?
            return sum;
        }

        long rsq(int a, int b) { return rsq(b) - rsq(a-1); }

        void point_update(int k, int val)	//O(log n), update = increment
        {
            while(k <= n) { ft[k] += val; k += k & -k; }		//min?
        }
    }
    static class Scanner {
        StringTokenizer st;
        BufferedReader br;

        public Scanner(FileReader r) {
            br = new BufferedReader(r);
        }

        public Scanner(InputStream s) {
            br = new BufferedReader(new InputStreamReader(s));
        }

        public String next() throws IOException {
            while (st == null || !st.hasMoreTokens())
                st = new StringTokenizer(br.readLine());
            return st.nextToken();
        }

        public int nextInt() throws IOException {
            return Integer.parseInt(next());
        }

        public long nextLong() throws IOException {
            return Long.parseLong(next());
        }

        public String nextLine() throws IOException {
            return br.readLine();
        }

        public double nextDouble() throws IOException {
            String x = next();
            StringBuilder sb = new StringBuilder("0");
            double res = 0, f = 1;
            boolean dec = false, neg = false;
            int start = 0;
            if (x.charAt(0) == '-') {
                neg = true;
                start++;
            }
            for (int i = start; i < x.length(); i++)
                if (x.charAt(i) == '.') {
                    res = Long.parseLong(sb.toString());
                    sb = new StringBuilder("0");
                    dec = true;
                } else {
                    sb.append(x.charAt(i));
                    if (dec)
                        f *= 10;
                }
            res += Long.parseLong(sb.toString()) / f;
            return res * (neg ? -1 : 1);
        }

        public boolean ready() throws IOException {
            return br.ready();
        }
    }

}