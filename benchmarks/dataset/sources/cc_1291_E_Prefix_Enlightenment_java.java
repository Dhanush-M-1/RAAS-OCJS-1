import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.PrintWriter;
import java.util.Arrays;
import java.util.InputMismatchException;
import java.util.Iterator;

public class Main {

	private static final String NO = "No";
	private static final String YES = "Yes";
	InputStream is;
	PrintWriter out;
	String INPUT = "";

	private static final long MOD = 1000000007;
	static int MAXN = 10;

	void solve() {
		int T = 1;// ni();

		for (int i = 0; i < T; i++)
			solve(i);
	}

	// index 1 based
	int fa[];
	int size[];

	void init(int n) {
		fa = new int[n + 1];
		size = new int[n + 1];
		for (int i = 1; i <= n; i++) {
			fa[i] = i;
			size[i] = i <= n / 2 ? 1 : 0;
		}
	}

	private void merge(int u, int v) {
		int u0 = find(u);
		int v0 = find(v);
		if (u0 == v0)
			return;
		fa[u0] = v0;
		size[v0] += size[u0];
	}

	private int find(int x) {
		if (fa[x] == x)
			return x;
		else
			return fa[x] = find(fa[x]);
	}

	void solve(int nth) {
		int n = ni(), m = ni();
		char[] s = ns().toCharArray();
		int c[][] = new int[n][2];
		for (int i = 1; i <= m; i++) {
			int x = ni();
			while (x-- > 0) {
				int a = ni() - 1;
				c[a][c[a][0] > 0 ? 1 : 0] = i;
			}
		}
		init(m + m + 1);
		fa[m * 2 + 1] = m * 2 + 1;
		size[m * 2 + 1] = (int) 1e9;
		int ans = 0;
		for (int i = 0; i < n; i++) {
			if (c[i][0] > 0 && c[i][1] == 0) {
				if (s[i] == '0') {// 必须取，那么将不取放入n*2+1的并查集中，意味着以后不会有不取这个选项
					ans -= Math.min(size[find(c[i][0])], size[find(c[i][0] + m)]);
					// 因为不知道前面的运算中，这个是取还是不取，但是一定是用最小的那个
					merge(2 * m + 1, c[i][0] + m);
					ans += size[find(c[i][0])];
				} else {// 同上类似，但是这次将“取”这个选项放入不可能并查集中
					ans -= Math.min(size[find(c[i][0])], size[find(c[i][0] + m)]);
					merge(2 * m + 1, c[i][0]);
					ans += size[find(c[i][0] + m)];
				}
			} else if (c[i][0] > 0 && c[i][1] > 0) {
				int x = c[i][0], y = c[i][1];
				if (s[i] == '0') {// 两个中取一个
					if (find(x) != find(y + m)) {
						ans -= Math.min(size[find(x)], size[find(x + m)]) + Math.min(size[find(y + m)], size[find(y)]);
						merge(x, y + m);
						merge(x + m, y);
						ans += Math.min(size[find(x)], size[find(y)]);
					}
				} else {
					if (find(x) != find(y)) {
						ans -= Math.min(size[find(x)], size[find(x + m)]) + Math.min(size[find(y + m)], size[find(y)]);
						merge(x, y);
						merge(x + m, y + m);
						ans += Math.min(size[find(x)], size[find(x + m)]);
					}
				}
			}
//			tr(i, ans, c[i], fa, mi);
			out.println(ans);
		}

	}

	// a^b
	long power(long a, long b) {
		long x = 1, y = a;
		while (b > 0) {
			if (b % 2 != 0) {
				x = (x * y) % MOD;
			}
			y = (y * y) % MOD;
			b /= 2;
		}
		return x % MOD;
	}

	private long gcd(long a, long b) {
		while (a != 0) {
			long tmp = b % a;
			b = a;
			a = tmp;
		}
		return b;
	}

	void run() throws Exception {
		is = INPUT.isEmpty() ? System.in : new ByteArrayInputStream(INPUT.getBytes());
		out = new PrintWriter(System.out);

		long s = System.currentTimeMillis();
		solve();
		out.flush();
		if (!INPUT.isEmpty())
			tr(System.currentTimeMillis() - s + "ms");
	}

	public static void main(String[] args) throws Exception {
		new Main().run();
	}

	private byte[] inbuf = new byte[1024];
	public int lenbuf = 0, ptrbuf = 0;

	private int readByte() {
		if (lenbuf == -1)
			throw new InputMismatchException();
		if (ptrbuf >= lenbuf) {
			ptrbuf = 0;
			try {
				lenbuf = is.read(inbuf);
			} catch (IOException e) {
				throw new InputMismatchException();
			}
			if (lenbuf <= 0)
				return -1;
		}
		return inbuf[ptrbuf++];
	}

	private boolean isSpaceChar(int c) {
		return !(c >= 33 && c <= 126);
	}

	private int skip() {
		int b;
		while ((b = readByte()) != -1 && isSpaceChar(b))
			;
		return b;
	}

	private double nd() {
		return Double.parseDouble(ns());
	}

	private char nc() {
		return (char) skip();
	}

	private char[] nc(int n) {
		char[] ret = new char[n];
		for (int i = 0; i < n; i++)
			ret[i] = nc();
		return ret;
	}

	private String ns() {
		int b = skip();
		StringBuilder sb = new StringBuilder();
		while (!(isSpaceChar(b))) { // when nextLine, (isSpaceChar(b) && b != '
									// ')
			sb.appendCodePoint(b);
			b = readByte();
		}
		return sb.toString();
	}

	private char[] ns(int n) {
		char[] buf = new char[n];
		int b = skip(), p = 0;
		while (p < n) {
			if (!(isSpaceChar(b)))
				buf[p++] = (char) b;
			b = readByte();
		}
		return n == p ? buf : Arrays.copyOf(buf, p);
	}

	private char[][] nm(int n, int m) {
		char[][] map = new char[n][];
		for (int i = 0; i < n; i++)
			map[i] = ns(m);
		return map;
	}

	private int[] na(int n) {
		int[] a = new int[n];
		for (int i = 0; i < n; i++)
			a[i] = ni();
		return a;
	}

	private Integer[] na2(int n) {
		Integer[] a = new Integer[n];
		for (int i = 0; i < n; i++)
			a[i] = ni();
		return a;
	}

	private int[][] na(int n, int m) {
		int[][] a = new int[n][];
		for (int i = 0; i < n; i++)
			a[i] = na(m);
		return a;
	}

	private int ni() {
		int num = 0, b;
		boolean minus = false;
		while ((b = readByte()) != -1 && !((b >= '0' && b <= '9') || b == '-'))
			;
		if (b == '-') {
			minus = true;
			b = readByte();
		}

		while (true) {
			if (b >= '0' && b <= '9') {
				num = num * 10 + (b - '0');
			} else {
				return minus ? -num : num;
			}
			b = readByte();
		}
	}

	private Long[] nl2(int n) {
		Long[] a = new Long[n];
		for (int i = 0; i < n; i++)
			a[i] = nl();
		return a;
	}

	private long[] nl(int n) {
		long[] a = new long[n];
		for (int i = 0; i < n; i++)
			a[i] = nl();
		return a;
	}

	private long[][] nl(int n, int m) {
		long[][] a = new long[n][];
		for (int i = 0; i < n; i++)
			a[i] = nl(m);
		return a;
	}

	private long nl() {
		long num = 0;
		int b;
		boolean minus = false;
		while ((b = readByte()) != -1 && !((b >= '0' && b <= '9') || b == '-'))
			;
		if (b == '-') {
			minus = true;
			b = readByte();
		}

		while (true) {
			if (b >= '0' && b <= '9') {
				num = num * 10 + (b - '0');
			} else {
				return minus ? -num : num;
			}
			b = readByte();
		}
	}

	public static class IterablePermutation implements Iterable<int[]>, Iterator<int[]> {
		int[] a;
		boolean first = true;

		public IterablePermutation(int n) {
			assert n >= 1;
			a = new int[n];
			for (int i = 0; i < n; i++)
				a[i] = i;
		}

		public IterablePermutation(int... a) {
			this.a = Arrays.copyOf(a, a.length);
		}

		@Override
		public boolean hasNext() {
			if (first)
				return true;
			int n = a.length;
			int i;
			for (i = n - 2; i >= 0 && a[i] >= a[i + 1]; i--)
				;
			return i != -1;
		}

		@Override
		public int[] next() {
			if (first) {
				first = false;
				return a;
			}
			int n = a.length;
			int i;
			for (i = n - 2; i >= 0 && a[i] >= a[i + 1]; i--)
				;
			assert i != -1;
			int j;
			for (j = i + 1; j < n && a[i] < a[j]; j++)
				;
			int d = a[i];
			a[i] = a[j - 1];
			a[j - 1] = d;
			for (int p = i + 1, q = n - 1; p < q; p++, q--) {
				d = a[p];
				a[p] = a[q];
				a[q] = d;
			}
			return a;
		}

		@Override
		public Iterator<int[]> iterator() {
			return this;
		}
	}

	private static void tr(Object... o) {
		System.out.println(Arrays.deepToString(o));
	}

}
