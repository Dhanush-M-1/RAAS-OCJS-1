import java.io.*;
import java.util.ArrayList;
import java.util.StringTokenizer;


public class Main {
    public static void main(String[] args) {
        InputStream inputStream = System.in;
        OutputStream outputStream = System.out;
        InputReader inp = new InputReader(inputStream);
        PrintWriter out = new PrintWriter(outputStream);
        Solver solver = new Solver();
        solver.solve(inp, out);
        out.close();
    }

    private static class Solver {
        private void solve(InputReader inp, PrintWriter out) {
            int n = inp.nextInt();

            ArrayList<Integer>[] tree = new ArrayList[n];
            for (int i = 0; i < n; i++) tree[i] = new ArrayList<>();
            for (int i = 0; i < n - 1; i++) {
                int u = inp.nextInt() - 1, v = inp.nextInt() - 1;
                tree[u].add(v);
                tree[v].add(u);
            }

            for (int i = 0; i < n; i++) {
                if (tree[i].size() == 2) {
                    out.print("NO");
                    return;
                }
            }
            out.print("YES");
        }
    }

    static class InputReader {
        BufferedReader reader;
        StringTokenizer tokenizer;

        InputReader(InputStream stream) {
            reader = new BufferedReader(new InputStreamReader(stream), 32768);
            tokenizer = null;
        }

        String next() {
            while (tokenizer == null || !tokenizer.hasMoreTokens()) {
                try {
                    tokenizer = new StringTokenizer(reader.readLine());
                } catch (IOException e) {
                    throw new RuntimeException(e);
                }
            }
            return tokenizer.nextToken();
        }

        public int nextInt() {
            return Integer.parseInt(next());
        }
        public long nextLong() {
            return Long.parseLong(next());
        }
    }
}