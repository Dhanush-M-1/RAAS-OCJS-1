import java.util.Scanner;
import java.io.OutputStream;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.ArrayList;
import java.io.InputStream;

/**
 * Built using CHelper plug-in
 * Actual solution is at the top
 */
public class Main {
	public static void main(String[] args) {
		InputStream inputStream = System.in;
		OutputStream outputStream = System.out;
		Scanner in = new Scanner(inputStream);
		PrintWriter out = new PrintWriter(outputStream);
		TaskB solver = new TaskB();
		solver.solve(1, in, out);
		out.close();
	}
}

class TaskB {
    public void solve(@SuppressWarnings("UnusedParameters") int testNumber, Scanner in, PrintWriter out) {
        int n = in.nextInt(), r = in.nextInt(), c = in.nextInt();
        String s;
        do {
            s = in.nextLine().trim();
        } while (s.isEmpty());
        ArrayList<Integer> spaces = new ArrayList<Integer>();
        spaces.add(-1);
        for (int i = 0; i < s.length(); ++i)
            if (s.charAt(i) == ' ')
                spaces.add(i);
        spaces.add(s.length());
        String[] w = new String[spaces.size() - 1];
        for (int i = 0; i < w.length; ++i)
            w[i] = s.substring(spaces.get(i) + 1, spaces.get(i+1));
/*
        String[] w = new String[n];
        for (int i = 0; i < n; ++i) {
            w[i] = in.next();
        }
*/
        int[][] step = new int[21][n];
        int j = 0, cur = 0;
        for (int i = 0; i < n; ++i) {
            while (j < n && cur + w[j].length() + 1 <= c+1) {
                cur += w[j].length() + 1;
                ++j;
            }
            step[0][i] = j;
            if (j > i)
                cur -= (w[i].length() + 1);
            else
                ++j;
        }

        for (int k = 1; k < step.length; ++k) {
            for (int i = 0; i < n; ++i) {
                int next = step[k-1][i];
                if (next == n)
                    step[k][i] = n;
                else
                    step[k][i] = step[k-1][next];
            }
        }

        int bestI = -1, bestCount = -1;
        for (int i = 0; i < n; ++i) {
            cur = i;
            for (int k = 0; k < step.length; ++k)
                if (0 != (r & (1<<k)) && cur < n) {
                    cur = step[k][cur];
                }
            int curCount = cur - i;
            if (curCount > bestCount) {
                bestCount = curCount;
                bestI = i;
            }
        }

        cur = 0;
        for (int i = bestI; i < bestI + bestCount; ++i) {
            if (cur + w[i].length() + 1 > c+1) {
                out.println();
                cur = 0;
            }
            if (cur > 0)
                out.print(" ");
            out.print(w[i]);
            cur += w[i].length() + 1;
        }
        out.println();
    }
}

