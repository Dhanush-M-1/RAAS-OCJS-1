import java.util.*;
import java.io.*;

public class Main {
	static int[][] dp;
	static int mod = (int) 1e9 + 7;

	public static int dp(int rem, int dig) {
		if (rem == 0) {
			return 1;
		}
		if (dp[dig][rem] != -1) {
			return dp[dig][rem];
		}
		if (dig == 9) {
			return dp[dig][rem] = (dp(rem - 1, 0) + dp(rem - 1, 1)) % mod;
		}
		return dp[dig][rem] = dp(rem - 1, dig + 1);
	}

	public static void main(String[] args) throws IOException {
		Scanner sc = new Scanner(System.in);
		PrintWriter pw = new PrintWriter(System.out);
		dp = new int[10][(int) 2e5 + 1];
		for (int[] x : dp)
			Arrays.fill(x, -1);
		int t = sc.nextInt();
		while (t-- > 0) {
			int n = sc.nextInt();
			int m = sc.nextInt();
			int ans = 0;
			while (n > 0) {
				ans = (ans + dp(m, n % 10)) % mod;
				n /= 10;
			}
			pw.println(ans);
		}
		pw.close();
	}

	static class Scanner {
		BufferedReader br;
		StringTokenizer st;

		public Scanner(InputStream s) {
			br = new BufferedReader(new InputStreamReader(s));
		}

		public Scanner(FileReader f) {
			br = new BufferedReader(f);
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

		public double nextDouble() throws IOException {
			return Double.parseDouble(next());
		}

		public int[] nextIntArr(int n) throws IOException {
			int[] arr = new int[n];
			for (int i = 0; i < n; i++) {
				arr[i] = Integer.parseInt(next());
			}
			return arr;
		}

	}

}
