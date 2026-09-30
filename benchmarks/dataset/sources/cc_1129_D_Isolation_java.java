import java.io.*;
import java.util.*;
 
public class Main {
	static final int MD = 998244353, A = 100000, B = 500;
	static int[] bb, dp, ss;
	static int[][] dq;
	static void update(int h) {
		int[] qq = dq[h];
		Arrays.fill(qq, 0);
		int t = 0;
		for (int i = (h + 1) * B; i > h * B; i--) {
			t += bb[i];
			qq[B + t] = (qq[B + t] + dp[i - 1]) % MD;
		}
		for (int c = 1; c <= B + B; c++)
			qq[c] = (qq[c] + qq[c - 1]) % MD;
	}
	public static void main(String[] args) throws IOException {
		BufferedReader br = new BufferedReader(new InputStreamReader(System.in));
		StringTokenizer st = new StringTokenizer(br.readLine());
		int n = Integer.parseInt(st.nextToken());
		int k = Integer.parseInt(st.nextToken());
		st = new StringTokenizer(br.readLine());
		int[] pp = new int[1 + n];
		int[] ii = new int[1 + A];
		for (int i = 1; i <= n; i++) {
			int a = Integer.parseInt(st.nextToken());
			pp[i] = ii[a]; ii[a] = i;
		}
		bb = new int[1 + n];
		dp = new int[1 + n];
		dp[0] = 1;
		int m = (n + B - 1) / B;
		ss = new int[m];
		dq = new int[m][B + 1 + B];
		for (int j = 1; j <= n; j++) {
			int p;
			m = (j - 1) / B;
			ss[m] += 1 - bb[j]; bb[j] = 1;
			if ((p = pp[j]) != 0) {
				int h = (p - 1) / B;
				ss[h] += -1 - bb[p]; bb[p] = -1;
				if (p <= m * B)
					update(h);
				if ((p = pp[p]) != 0) {
					h = (p - 1) / B;
					ss[h] += 0 - bb[p]; bb[p] = 0;
					if (p <= m * B)
						update(h);
				}
			}
			int x = 0, t = 0;
			for (int i = j; i > m * B; i--)
				if ((t += bb[i]) <= k)
					x = (x + dp[i - 1]) % MD;
			for (int h = m - 1; h >= 0; h--) {
				if (k - t >= -B)
					x = (x + dq[h][B + Math.min(B, k - t)]) % MD;
				t += ss[h];
			}
			dp[j] = x;
			if (j % B == 0)
				update(m);
		}
		System.out.println(dp[n]);
	}
}