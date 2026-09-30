import java.io.*;
import java.util.*;
 
public class Main extends PrintWriter {
    
    private void solve() {
        int T = sc.nextInt();
        for(int tc = 1; tc <= T; tc++) {
            n = sc.nextInt();
            k = sc.nextInt();
            
            minions = new long[n][];
            for(int i = 0; i < n; i++) {
                minions[i] = new long[] {sc.nextLong(), sc.nextLong(), i+1};
            }
            Arrays.sort(minions, Comparator.comparing(arr->arr[1]));
            memo = new Long[n][k+1];
            
            long[][] dp = new long[n+1][k+1];
            int[][] choice = new int[n+1][k+1];
            for(int i = 0; i <= n; i++) Arrays.fill(dp[i], -1L<<60);
            dp[n][k] = 0L;
            for(int i = n-1; i >= 0; i--) {
                for(int j = k; j >= 0; j--) {
                    if(j+1 <= k) {
                        long cost = minions[i][0] + j*minions[i][1] + dp[i+1][j+1];
                        if(cost > dp[i][j]) {
                            dp[i][j] = cost;
                            choice[i][j] = 0;
                        }
                    }
                    long cost = (k-1)*minions[i][1] + dp[i+1][j];
                    if(cost > dp[i][j]) {
                        dp[i][j] = cost;
                        choice[i][j] = 1;
                    }
                }
            }
            int idx = 0;
            int sz = 0;
            ArrayDeque<Integer> ans1 = new ArrayDeque<Integer>();
            ArrayDeque<Integer> ans2 = new ArrayDeque<Integer>();
            int last = -1;
            while(idx < n) {
                int num = (int)minions[idx][2];
                if(choice[idx][sz] == 0) {
                    sz++;
                    if(sz == k) {
                        last = num;
                    } else {
                        ans1.add(num);
                    }
                } else {
                    ans2.add(num);
                    ans2.add(-num);
                }
                idx++;
            }
            println(ans1.size() + ans2.size() + 1);
            for(int x : ans1) {
                print(x + " ");
            }
            for(int x : ans2) {
                print(x + " ");
            }
            println(last);
        }
    }
    
    int n;
    int k;
    
    long[][] minions;
    Long[][] memo;
    
    long doit(int i, int j) {
        if(i == n && j == k) return 0L;
        else if(j > k || i == n) return -1L<<60;
        else if(memo[i][j] == null) {
            long res = 0L;
            res = Math.max(res, minions[i][0] + j*minions[i][1] + doit(i+1,j+1));
            res = Math.max(res, (k-1)*minions[i][1] + doit(i+1,j));
            memo[i][j] = res;
        }
        return memo[i][j];
    }
    
    
    
//  Main() throws FileNotFoundException { super(new File("output.txt")); }
//  InputReader sc = new InputReader(new FileInputStream("test_input.txt"));
    
  Main() { super(System.out); }
  InputReader sc = new InputReader(System.in);
  static class InputReader {
      InputReader(InputStream in) { this.in = in; } InputStream in;
      
      private byte[] buf = new byte[16384];
      private int    curChar;
      private int    numChars;
      
 
      public int read() {
          if (numChars == -1)
              throw new InputMismatchException();
          if (curChar >= numChars) {
              curChar = 0;
              try {
                  numChars = in.read(buf);
              } catch (IOException e) {
                  throw new InputMismatchException();
              }
              if (numChars <= 0)
                  return -1;
          }
          return buf[curChar++];
      }
 
      public String nextLine() {
          int c = read();
          while (isSpaceChar(c))
              c = read();
          StringBuilder res = new StringBuilder();
          do {
              res.appendCodePoint(c);
              c = read();
          } while (!isEndOfLine(c));
          return res.toString();
      }
 
      public String nextString() {
          int c = read();
          while (isSpaceChar(c))
              c = read();
          StringBuilder res = new StringBuilder();
          do {
              res.appendCodePoint(c);
              c = read();
          } while (!isSpaceChar(c));
          return res.toString();
      }
 
      public long nextLong() {
          int c = read();
          while (isSpaceChar(c))
              c = read();
          int sgn = 1;
          if (c == '-') {
              sgn = -1;
              c = read();
          }
          long res = 0;
          do {
              if (c < '0' || c > '9')
                  throw new InputMismatchException();
              res *= 10;
              res += c - '0';
              c = read();
          } while (!isSpaceChar(c));
          return res * sgn;
      }
 
      public int nextInt() {
          int c = read();
          while (isSpaceChar(c))
              c = read();
          int sgn = 1;
          if (c == '-') {
              sgn = -1;
              c = read();
          }
          int res = 0;
          do {
              if (c < '0' || c > '9')
                  throw new InputMismatchException();
              res *= 10;
              res += c - '0';
              c = read();
          } while (!isSpaceChar(c));
          return res * sgn;
      }
 
      private boolean isSpaceChar(int c) {
          return c == ' ' || c == '\n' || c == '\r' || c == '\t' || c == -1;
      }
 
      private boolean isEndOfLine(int c) {
          return c == '\n' || c == '\r' || c == -1;
      }
  }
 
    public static void main(String[] $) {
        new Thread(null, new Runnable() {
            public void run() {
                long start = System.nanoTime();
                try {Main solution = new Main(); solution.solve(); solution.close();} 
                catch (Exception e) {e.printStackTrace(); System.exit(1);}
                System.err.println((System.nanoTime()-start)/1E9);
            }
        }, "1", 1 << 27).start();
 
    }
    
}