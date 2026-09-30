

import javax.swing.tree.TreeNode;
import java.io.IOException;
import java.io.InputStream;
import java.util.*;

public class Main {


    static final class InputReader {
        private final InputStream stream;
        private final byte[] buf = new byte[1024];
        private int curChar;
        private int numChars;

        public InputReader(InputStream stream) {
            this.stream = stream;
        }

        private int read() throws IOException {
            if (curChar >= numChars) {
                curChar = 0;
                numChars = stream.read(buf);
                if (numChars <= 0) {
                    return -1;
                }
            }
            return buf[curChar++];
        }

        public final int readInt() throws IOException {
            return (int) readLong();
        }

        public final long readLong() throws IOException {
            int c = read();
            while (isSpaceChar(c)) {
                c = read();
                if (c == -1) throw new IOException();
            }
            boolean negative = false;
            if (c == '-') {
                negative = true;
                c = read();
            }
            long res = 0;
            do {
                res *= 10;
                res += c - '0';
                c = read();
            } while (!isSpaceChar(c));
            return negative ? -res : res;
        }

        public final int[] readIntArray(int size) throws IOException {
            int[] array = new int[size];
            for (int i = 0; i < size; i++) {
                array[i] = readInt();
            }
            return array;
        }

        public final long[] readLongArray(int size) throws IOException {
            long[] array = new long[size];
            for (int i = 0; i < size; i++) {
                array[i] = readLong();
            }
            return array;
        }

        public final String readString() throws IOException {
            int c = read();
            while (isSpaceChar(c)) {
                c = read();
            }
            StringBuilder res = new StringBuilder();
            do {
                res.append((char) c);
                c = read();
            } while (!isSpaceChar(c));
            return res.toString();
        }

        private boolean isSpaceChar(int c) {
            return c == ' ' || c == '\n' || c == '\r' || c == '\t' || c == -1;
        }
    }

    static long mulmod(long a, long b,
                       long mod) {
        long res = 0; // Initialize result
        a = a % mod;
        while (b > 0) {
            // If b is odd, add 'a' to result
            if (b % 2 == 1) {
                res = (res + a) % mod;
            }

            // Multiply 'a' with 2
            a = (a * 2) % mod;

            // Divide b by 2
            b /= 2;
        }

        // Return result
        return res % mod;
    }

    static long pow(long a, long b, long MOD) {
        long x = 1, y = a;
        while (b > 0) {
            if (b % 2 == 1) {
                x = (x * y);
                if (x > MOD) x %= MOD;
            }
            y = (y * y);
            if (y > MOD) y %= MOD;
            b /= 2;
        }
        return x;
    }

    static long[] f = new long[100001];

    static long InverseEuler(long n, long MOD) {
        return pow(n, MOD - 2, MOD);
    }

    static long C(int n, int r, long MOD) {

        return (f[n] * ((InverseEuler(f[r], MOD) * InverseEuler(f[n - r], MOD)) % MOD)) % MOD;
    }


    static int[] h = {0, 0, -1, 1};
    static int[] v = {1, -1, 0, 0};


    public static class Pair {
        public int a;

        public int b;

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            Pair pair = (Pair) o;
            return a == pair.a &&
                    b == pair.b;
        }

        @Override
        public int hashCode() {
            return Objects.hash(a, b);
        }

        public Pair(int a, int b) {
            this.a = a;
            this.b = b;
        }
    }

    static class Pair2 {
        public long cost;
        int node;

        public Pair2(long cos, int node) {
            this.cost = cos;
            this.node = node;
        }
    }

    static long compute_hash(String s) {
        int p = 31;
        int m = 1000000007;
        long hash_value = 0;
        long p_pow = 1;
        for (int i = 0; i < s.length(); ++i) {
            char c = s.charAt(i);
            hash_value = (hash_value + (c - 'a' + 1) * p_pow) % m;
            p_pow = (p_pow * p) % m;
        }
        return hash_value;
    }

    public static class SegmentTree {
        long[][] tree;
        int n;

        public SegmentTree(int[] nodes) {
            tree = new long[nodes.length * 4][2];
            n = nodes.length;
            build(0, n - 1, 0, nodes);

        }

        private void build(int l, int r, int pos, int[] nodes) {
            if (l == r) {
                tree[pos][0] = nodes[l];
                tree[pos][1] = l;
                return;
            }
            int mid = (l + r) / 2;
            build(l, mid, 2 * pos + 1, nodes);
            build(mid + 1, r, 2 * pos + 2, nodes);
            if (tree[2 * pos + 1][0] > tree[2 * pos + 2][0]) {
                tree[pos][1] = tree[2 * pos + 1][1];
            } else {
                tree[pos][1] = tree[2 * pos + 2][1];
            }
            tree[pos][0] = Math.max(tree[2 * pos + 1][0], tree[2 * pos + 2][0]);
        }

//        public void update(int pos, int val) {
//            updateUtil(0, n - 1, 0, pos, val);
//        }

        public long[] get(int l, int r) {
            return getUtil(0, n - 1, 0, l, r);
        }

        private long[] getUtil(int l, int r, int pos, int ql, int qr) {
            if (ql > r || qr < l) {
                return new long[]{-1, -1};
            }
            if (l >= ql && r <= qr) {
                return tree[pos];
            }

            int mid = (l + r) / 2;
            long[] left = getUtil(l, mid, 2 * pos + 1, ql, qr);
            long[] right = getUtil(mid + 1, r, 2 * pos + 2, ql, qr);
            long choice = right[1];
            if (left[0] > right[0]) choice = left[1];
            return new long[]{Math.max(left[0], right[0]), choice};

        }

//        private void updateUtil(int l, int r, int pos, int i, int val) {
//            if (i < l || i > r) {
//                return;
//            }
//            if (l == r) {
//                tree[pos] = val;
//                return;
//            }
//            int mid = (l + r) / 2;
//            updateUtil(l, mid, 2 * pos + 1, i, val);
//            updateUtil(mid + 1, r, 2 * pos + 2, i, val);
//            tree[pos] = tree[2 * pos + 1] + tree[2 * pos + 2];
//        }
    }

    static int counter = 0;
    static int[] rIn;
    static int[] rOut;
    static int[] lIn;
    static int[] lOut;
    private static int[] flatten;
    private static int[] lFlatten;
    static long answer = 0;

    static int VISITED = 1;
    static int VISITING = 2;

    static int[] DIRX = new int[]{0, 0, 1, -1};
    static int[] DIRY = new int[]{1, -1, 0, 0};

    public static void main(String[] args) throws Exception {
        //https://i...content-available-to-author-only...e.com/ebRGa6
        InputReader in = new InputReader(System.in);


        int t = in.readInt();
        while (t-- > 0) {

            int n = in.readInt();

            int a = 0, b = 0, c = 0, d = 0;

            for (int i = 0; i < n; ++i) {
                String s = in.readString();
                if (i == 0) {
                    a = s.charAt(1) - '0';
                } else if (i == 1) {
                    b = s.charAt(0) - '0';
                }
                if (i == n-2) {
                    c = s.charAt(n-1) - '0';
                } else if (i == n-1) {
                    d = s.charAt(n-2) - '0';
                }
            }
            List<Integer> cx = new ArrayList<>();
            List<Integer> cy = new ArrayList<>();
//            System.out.println(a + "  " + b + " " + c + " " + d);
            if (a == b) {
                if (c == a) {
                    cx.add(n-2);
                    cy.add(n-1);
                }
                if (d == a) {
                    cx.add(n-1);
                    cy.add(n-2);
                }
            } else {
                if (c == d) {
                    if (c == a) {
                        cx.add(0);
                        cy.add(1);
                    }
                    if (c == b) {
                        cx.add(1);
                        cy.add(0);
                    }
                } else {
                    if (0 == a) {
                        cx.add(0);
                        cy.add(1);
                    }
                    if (0 == b) {
                        cx.add(1);
                        cy.add(0);
                    }

                    if (c == 1) {
                        cx.add(n-2);
                        cy.add(n-1);
                    }
                    if (d == 1) {
                        cx.add(n-1);
                        cy.add(n-2);
                    }
                }
            }

            System.out.println(cx.size());
            for (int i = 0;i < cx.size(); ++i) {
                System.out.println((cx.get(i) + 1) + " " + (cy.get(i) + 1));
            }


        }

    }

    private static int dfs(HashMap<Pair, TreeSet<Pair>> grid, int x, int y, int ti, HashSet<Pair> vis, int r, int startX, int startY) {
//        System.out.println(x + "  " + y);
        int taken = ti - Math.abs(startX - x) - Math.abs(startY - y);
        if (taken < 0) return 0;
        if (x < 0 || y < 0 || x > r || y > r) return 0;
        if (vis.contains(new Pair(x, y))) return 0;
        int max = 0;

        if (grid.containsKey(new Pair(x, y))) {
            TreeSet<Pair> times = grid.get(new Pair(x, y));
            for (Pair t : times) {
                if (t.a <= taken) {
                    max = Math.max(t.b, max);
                } else break;
            }
        }
        vis.add(new Pair(x, y));
        max = Math.max(dfs(grid, x + 1, y, ti, vis, r, startX, startY), max);
        max = Math.max(dfs(grid, x, y + 1, ti, vis, r, startX, startY), max);
        max = Math.max(dfs(grid, x - 1, y, ti, vis, r, startX, startY), max);
        max = Math.max(dfs(grid, x, y - 1, ti, vis, r, startX, startY), max);
        return max;
    }

    private static int solver(int[] nums, int pos, int[] dp) {
        if (pos >= nums.length) return 0;
        if (dp[pos] != Integer.MAX_VALUE) return dp[pos];
        int min = solver(nums, pos + 2, dp) + nums[pos];
        min = Math.min(solver(nums, pos + 3, dp) + nums[pos], min);
        if (pos + 1 < nums.length) min = Math.min(min, nums[pos] + nums[pos + 1] + solver(nums, pos + 3, dp));
        if (pos + 1 < nums.length) min = Math.min(min, nums[pos] + nums[pos + 1] + solver(nums, pos + 4, dp));

//        System.out.println(pos + " " + min);
        return dp[pos] = min;
    }


    static int countFreq(String pattern, String text) {
        int m = pattern.length();
        int n = text.length();
        int res = 0;

        for (int i = 0; i <= n - m; i++) {
            int j;
            for (j = 0; j < m; j++) {
                if (text.charAt(i + j) != pattern.charAt(j)) {
                    break;
                }
            }
            if (j == m) {
                res++;
                j = 0;
            }
        }
        return res;
    }

    private static int solve(int n, int[] dp, int cn) {
        if (cn > n) return 0;
        if (cn == n) return 1;
        if (dp[cn] != -1) return dp[cn];
        int ans = 0;
        for (int i = 2; i <= 9; ++i) {
            ans += solve(n, dp, cn * i);
        }
        return dp[cn] = ans;
    }


    private static void dfsR(List<List<Integer>> g, int node, int[] v) {
        rIn[node] = counter;
        flatten[counter++] = v[node];

        for (int i = 0; i < g.get(node).size(); ++i) {
            dfsR(g, g.get(node).get(i), v);
        }

        rOut[node] = counter;
        flatten[counter++] = v[node] * -1;
    }

    private static void dfsL(List<List<Integer>> g, int node, int[] v) {
        lIn[node] = counter;
        lFlatten[counter++] = v[node];

        for (int i = 0; i < g.get(node).size(); ++i) {
            dfsL(g, g.get(node).get(i), v);
        }

        lOut[node] = counter;
        lFlatten[counter++] = v[node] * -1;

        TreeMap<String, Integer> map = new TreeMap<>();
    }


    private static void preprocess(int pos, int[][] pre, List<List<Integer>> tree, int[] traverse, int depth, int last, int[] tin, int[] tout) {
        tin[pos] = counter++;
        traverse[depth] = pos;

        for (int i = 0; depth - (1 << i) >= 0; ++i) {
            pre[pos][i] = traverse[depth - (1 << i)];
        }

        for (int i = 0; i < tree.get(pos).size(); ++i) {
            if (tree.get(pos).get(i) != last)
                preprocess(tree.get(pos).get(i), pre, tree, traverse, depth + 1, pos, tin, tout);
        }
        tout[pos] = counter++;
    }

    static int gcd(int a, int b) {

        while (b != 0) {
            int t = a;
            a = b;
            b = t % b;
        }
        return a;
    }


    static boolean submit = true;

    static void debug(String s) {
        if (!submit)
            System.out.println(s);
    }

    static void debug(int s) {

        Stack<Integer> st = new Stack<>();
        if (!submit)
            System.out.println(s);


        /*

        4 2 3 _ 2 4 3 _
         */
    }



}