import java.io.OutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.Arrays;
import java.io.IOException;
import java.util.Random;
import java.util.ArrayList;
import java.io.UncheckedIOException;
import java.util.List;
import java.io.Closeable;
import java.io.Writer;
import java.io.OutputStreamWriter;
import java.io.InputStream;

/**
 * Built using CHelper plug-in
 * Actual solution is at the top
 */
public class Main {
    public static void main(String[] args) throws Exception {
        Thread thread = new Thread(null, new TaskAdapter(), "", 1 << 27);
        thread.start();
        thread.join();
    }

    static class TaskAdapter implements Runnable {
        @Override
        public void run() {
            InputStream inputStream = System.in;
            OutputStream outputStream = System.out;
            FastInput in = new FastInput(inputStream);
            FastOutput out = new FastOutput(outputStream);
            ECycleSort solver = new ECycleSort();
            solver.solve(1, in, out);
            out.close();
        }
    }

    static class ECycleSort {
        int n;
        int[] a;

        public void solve(int testNumber, FastInput in, FastOutput out) {
            n = in.readInt();
            int s = in.readInt();
            a = new int[n];
            for (int i = 0; i < n; i++) {
                a[i] = in.readInt();
            }
            int[] b = a.clone();
            Randomized.shuffle(b);
            Arrays.sort(b);
            int[] same = new int[n];
            int sum = 0;
            for (int i = 0; i < n; i++) {
                if (a[i] == b[i]) {
                    same[i] = 1;
                }
            }
            for (int x : same) {
                sum += x;
            }
            if (n - sum > s) {
                out.println(-1);
                return;
            }
            IntegerList permList = new IntegerList(n);
            for (int i = 0; i < n; i++) {
                if (same[i] == 0) {
                    permList.add(i);
                }
            }
            int[] perm = permList.toArray();
            CompareUtils.quickSort(perm, (x, y) -> Integer.compare(a[x], a[y]), 0, perm.length);
            DSU dsu = new DSU(n);
            for (int i = 0; i < perm.length; i++) {
                int from = perm[i];
                int to = permList.get(i);
                dsu.merge(from, to);
            }
            for (int i = 1; i < perm.length; i++) {
                if (a[perm[i]] != a[perm[i - 1]]) {
                    continue;
                }
                if (dsu.find(perm[i]) == dsu.find(perm[i - 1])) {
                    continue;
                }
                dsu.merge(perm[i], perm[i - 1]);
                SequenceUtils.swap(perm, i, i - 1);
            }
            IntegerList first = new IntegerList();
            if (perm.length > 0) {
                int remain = s - (n - sum) - 1;
                first.add(perm[0]);
                for (int i = 1; remain > 0 && i < perm.length; i++) {
                    if (dsu.find(perm[i - 1]) != dsu.find(perm[i])) {
                        remain--;
                        first.add(perm[i]);
                        dsu.merge(perm[i - 1], perm[i]);
                    }
                }

                int last = a[first.get(0)];
                for (int i = 1; i < first.size(); i++) {
                    int y = first.get(i);
                    int tmp = a[y];
                    a[y] = last;
                    last = tmp;
                }
                a[first.get(0)] = last;
                //System.err.println(Arrays.toString(a));
            }

            List<IntegerList> circles = new ArrayList<>();
            if (first.size() > 1) {
                circles.add(first);
            }
            circles.addAll(solve());
            out.println(circles.size());
            for (IntegerList list : circles) {
                out.println(list.size());
                for (int i = 0; i < list.size(); i++) {
                    out.append(list.get(i) + 1).append(' ');
                }
                out.println();
            }
        }

        public List<IntegerList> solve() {
            int[] b = a.clone();
            Randomized.shuffle(b);
            Arrays.sort(b);
            int[] same = new int[n];
            for (int i = 0; i < n; i++) {
                if (a[i] == b[i]) {
                    same[i] = 1;
                }
            }
            IntegerList permList = new IntegerList(n);
            for (int i = 0; i < n; i++) {
                if (same[i] == 0) {
                    permList.add(i);
                }
            }
            int[] perm = permList.toArray();
            CompareUtils.quickSort(perm, (x, y) -> Integer.compare(a[x], a[y]), 0, perm.length);
            DSU dsu = new DSU(n);
            for (int i = 0; i < perm.length; i++) {
                int from = perm[i];
                int to = permList.get(i);
                dsu.merge(from, to);
            }
            for (int i = 1; i < perm.length; i++) {
                if (a[perm[i]] != a[perm[i - 1]]) {
                    continue;
                }
                if (dsu.find(perm[i]) == dsu.find(perm[i - 1])) {
                    continue;
                }
                dsu.merge(perm[i], perm[i - 1]);
                SequenceUtils.swap(perm, i, i - 1);
            }

            int[] index = new int[n];
            for (int i = 0; i < n; i++) {
                if (same[i] == 1) {
                    index[i] = i;
                }
            }
            for (int i = 0; i < perm.length; i++) {
                index[perm[i]] = permList.get(i);
            }
            PermutationUtils.PowerPermutation pp = new PermutationUtils.PowerPermutation(index);
            List<IntegerList> circles = pp.extractCircles(2);

            return circles;
        }

    }

    static class Randomized {
        private static Random random = new Random(0);

        public static void shuffle(int[] data) {
            shuffle(data, 0, data.length - 1);
        }

        public static void shuffle(int[] data, int from, int to) {
            to--;
            for (int i = from; i <= to; i++) {
                int s = nextInt(i, to);
                int tmp = data[i];
                data[i] = data[s];
                data[s] = tmp;
            }
        }

        public static int nextInt(int l, int r) {
            return random.nextInt(r - l + 1) + l;
        }

    }

    static class SequenceUtils {
        public static void swap(int[] data, int i, int j) {
            int tmp = data[i];
            data[i] = data[j];
            data[j] = tmp;
        }

        public static boolean equal(int[] a, int al, int ar, int[] b, int bl, int br) {
            if ((ar - al) != (br - bl)) {
                return false;
            }
            for (int i = al, j = bl; i <= ar; i++, j++) {
                if (a[i] != b[j]) {
                    return false;
                }
            }
            return true;
        }

    }

    static class FastInput {
        private final InputStream is;
        private byte[] buf = new byte[1 << 20];
        private int bufLen;
        private int bufOffset;
        private int next;

        public FastInput(InputStream is) {
            this.is = is;
        }

        private int read() {
            while (bufLen == bufOffset) {
                bufOffset = 0;
                try {
                    bufLen = is.read(buf);
                } catch (IOException e) {
                    bufLen = -1;
                }
                if (bufLen == -1) {
                    return -1;
                }
            }
            return buf[bufOffset++];
        }

        public void skipBlank() {
            while (next >= 0 && next <= 32) {
                next = read();
            }
        }

        public int readInt() {
            int sign = 1;

            skipBlank();
            if (next == '+' || next == '-') {
                sign = next == '+' ? 1 : -1;
                next = read();
            }

            int val = 0;
            if (sign == 1) {
                while (next >= '0' && next <= '9') {
                    val = val * 10 + next - '0';
                    next = read();
                }
            } else {
                while (next >= '0' && next <= '9') {
                    val = val * 10 - next + '0';
                    next = read();
                }
            }

            return val;
        }

    }

    static class FastOutput implements AutoCloseable, Closeable, Appendable {
        private StringBuilder cache = new StringBuilder(10 << 20);
        private final Writer os;

        public FastOutput append(CharSequence csq) {
            cache.append(csq);
            return this;
        }

        public FastOutput append(CharSequence csq, int start, int end) {
            cache.append(csq, start, end);
            return this;
        }

        public FastOutput(Writer os) {
            this.os = os;
        }

        public FastOutput(OutputStream os) {
            this(new OutputStreamWriter(os));
        }

        public FastOutput append(char c) {
            cache.append(c);
            return this;
        }

        public FastOutput append(int c) {
            cache.append(c);
            return this;
        }

        public FastOutput println(int c) {
            cache.append(c);
            println();
            return this;
        }

        public FastOutput println() {
            cache.append(System.lineSeparator());
            return this;
        }

        public FastOutput flush() {
            try {
                os.append(cache);
                os.flush();
                cache.setLength(0);
            } catch (IOException e) {
                throw new UncheckedIOException(e);
            }
            return this;
        }

        public void close() {
            flush();
            try {
                os.close();
            } catch (IOException e) {
                throw new UncheckedIOException(e);
            }
        }

        public String toString() {
            return cache.toString();
        }

    }

    static class IntegerList implements Cloneable {
        private int size;
        private int cap;
        private int[] data;
        private static final int[] EMPTY = new int[0];

        public IntegerList(int cap) {
            this.cap = cap;
            if (cap == 0) {
                data = EMPTY;
            } else {
                data = new int[cap];
            }
        }

        public IntegerList(IntegerList list) {
            this.size = list.size;
            this.cap = list.cap;
            this.data = Arrays.copyOf(list.data, size);
        }

        public IntegerList() {
            this(0);
        }

        public void ensureSpace(int req) {
            if (req > cap) {
                while (cap < req) {
                    cap = Math.max(cap + 10, 2 * cap);
                }
                data = Arrays.copyOf(data, cap);
            }
        }

        private void checkRange(int i) {
            if (i < 0 || i >= size) {
                throw new ArrayIndexOutOfBoundsException();
            }
        }

        public int get(int i) {
            checkRange(i);
            return data[i];
        }

        public void add(int x) {
            ensureSpace(size + 1);
            data[size++] = x;
        }

        public void addAll(int[] x, int offset, int len) {
            ensureSpace(size + len);
            System.arraycopy(x, offset, data, size, len);
            size += len;
        }

        public void addAll(IntegerList list) {
            addAll(list.data, 0, list.size);
        }

        public int size() {
            return size;
        }

        public int[] toArray() {
            return Arrays.copyOf(data, size);
        }

        public String toString() {
            return Arrays.toString(toArray());
        }

        public boolean equals(Object obj) {
            if (!(obj instanceof IntegerList)) {
                return false;
            }
            IntegerList other = (IntegerList) obj;
            return SequenceUtils.equal(data, 0, size - 1, other.data, 0, other.size - 1);
        }

        public int hashCode() {
            int h = 1;
            for (int i = 0; i < size; i++) {
                h = h * 31 + Integer.hashCode(data[i]);
            }
            return h;
        }

        public IntegerList clone() {
            IntegerList ans = new IntegerList(size);
            ans.addAll(this);
            return ans;
        }

    }

    static class DSU {
        int[] p;
        int[] rank;

        public DSU(int n) {
            p = new int[n];
            rank = new int[n];
            reset();
        }

        public void reset() {
            for (int i = 0; i < p.length; i++) {
                p[i] = i;
                rank[i] = 0;
            }
        }

        public int find(int a) {
            return p[a] == p[p[a]] ? p[a] : (p[a] = find(p[a]));
        }

        public void merge(int a, int b) {
            a = find(a);
            b = find(b);
            if (a == b) {
                return;
            }
            if (rank[a] == rank[b]) {
                rank[a]++;
            }
            if (rank[a] > rank[b]) {
                p[b] = a;
            } else {
                p[a] = b;
            }
        }

    }

    static interface IntComparator {
        public int compare(int a, int b);

    }

    static class PermutationUtils {
        private static final long[] PERMUTATION_CNT = new long[21];

        static {
            PERMUTATION_CNT[0] = 1;
            for (int i = 1; i <= 20; i++) {
                PERMUTATION_CNT[i] = PERMUTATION_CNT[i - 1] * i;
            }
        }

        public static class PowerPermutation {
            int[] g;
            int[] idx;
            int[] l;
            int[] r;
            int n;

            public List<IntegerList> extractCircles(int threshold) {
                List<IntegerList> ans = new ArrayList<>(n);
                for (int i = 0; i < n; i = r[i] + 1) {
                    int size = r[i] - l[i] + 1;
                    if (size < threshold) {
                        continue;
                    }
                    IntegerList list = new IntegerList(r[i] - l[i] + 1);
                    for (int j = l[i]; j <= r[i]; j++) {
                        list.add(g[j]);
                    }
                    ans.add(list);
                }
                return ans;
            }

            public PowerPermutation(int[] p) {
                this(p, p.length);
            }

            public PowerPermutation(int[] p, int len) {
                n = len;
                boolean[] visit = new boolean[n];
                g = new int[n];
                l = new int[n];
                r = new int[n];
                idx = new int[n];
                int wpos = 0;
                for (int i = 0; i < n; i++) {
                    int val = p[i];
                    if (visit[val]) {
                        continue;
                    }
                    visit[val] = true;
                    g[wpos] = val;
                    l[wpos] = wpos;
                    idx[val] = wpos;
                    wpos++;
                    while (true) {
                        int x = p[g[wpos - 1]];
                        if (visit[x]) {
                            break;
                        }
                        visit[x] = true;
                        g[wpos] = x;
                        l[wpos] = l[wpos - 1];
                        idx[x] = wpos;
                        wpos++;
                    }
                    for (int j = l[wpos - 1]; j < wpos; j++) {
                        r[j] = wpos - 1;
                    }
                }
            }

            public int apply(int x, int p) {
                int i = idx[x];
                int dist = DigitUtils.mod((i - l[i]) + p, r[i] - l[i] + 1);
                return g[dist + l[i]];
            }

            public String toString() {
                StringBuilder builder = new StringBuilder();
                for (int i = 0; i < n; i++) {
                    builder.append(apply(i, 1)).append(' ');
                }
                return builder.toString();
            }

        }

    }

    static class CompareUtils {
        private static final int THRESHOLD = 4;

        private CompareUtils() {
        }

        public static <T> void insertSort(int[] data, IntComparator cmp, int l, int r) {
            for (int i = l + 1; i <= r; i++) {
                int j = i;
                int val = data[i];
                while (j > l && cmp.compare(data[j - 1], val) > 0) {
                    data[j] = data[j - 1];
                    j--;
                }
                data[j] = val;
            }
        }

        public static void quickSort(int[] data, IntComparator cmp, int f, int t) {
            if (t - f <= THRESHOLD) {
                insertSort(data, cmp, f, t - 1);
                return;
            }
            SequenceUtils.swap(data, f, Randomized.nextInt(f, t - 1));
            int l = f;
            int r = t;
            int m = l + 1;
            while (m < r) {
                int c = cmp.compare(data[m], data[l]);
                if (c == 0) {
                    m++;
                } else if (c < 0) {
                    SequenceUtils.swap(data, l, m);
                    l++;
                    m++;
                } else {
                    SequenceUtils.swap(data, m, --r);
                }
            }
            quickSort(data, cmp, f, l);
            quickSort(data, cmp, m, t);
        }

    }

    static class DigitUtils {
        private DigitUtils() {
        }

        public static int mod(int x, int mod) {
            x %= mod;
            if (x < 0) {
                x += mod;
            }
            return x;
        }

    }
}

