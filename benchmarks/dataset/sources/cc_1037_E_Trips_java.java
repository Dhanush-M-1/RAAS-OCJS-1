import java.util.*;
import java.io.*;

public class Main {

  private static void solve() {
    int n = nextInt();
    int m = nextInt();

    int k = nextInt();

    Map<Integer, Set<Integer>> friends = new HashMap<>();

    int[] f = new int[m];
    int[] t = new int[m];
    for (int i = 0; i < m; i++) {
      int a = nextInt() - 1;
      int b = nextInt() - 1;

      friends.computeIfAbsent(a, x -> new HashSet<>()).add(b);
      friends.computeIfAbsent(b, x -> new HashSet<>()).add(a);
      friends.get(a).add(b);
      friends.get(b).add(a);

      f[i] = a; t[i] = b;
    }

    for (int i = 0; i < n; i++) {
      remove(friends, i, k);
    }

    int[] ans = new int[m];
    ans[m - 1] = friends.size();
    for (int i = m - 1; i > 0; i--) {
      if (friends.get(f[i]) != null)
        friends.get(f[i]).remove(t[i]);
      if (friends.get(t[i]) != null)
        friends.get(t[i]).remove(f[i]);
      remove(friends, f[i], k);
      remove(friends, t[i], k);
      ans[i - 1] = friends.size();
    }


    for (int i = 0; i < m; i++) {
      out.println(ans[i]);
    }
  }

  private static void remove(Map<Integer, Set<Integer>> friends, int toRemove, int k) {
    if (friends.get(toRemove) == null || friends.get(toRemove).size() >= k) {
      return;
    }

    Set<Integer> list = friends.remove(toRemove);

    if (list != null) {
      for (Integer l : list) {
        if (friends.get(l) != null) {
          friends.get(l).remove(toRemove);
          remove(friends, l, k);
        }
      }
    }
  }


  private static void run() {
    br = new BufferedReader(new InputStreamReader(System.in));
    out = new PrintWriter(System.out);

    solve();

    out.close();
  }

  private static StringTokenizer st;
  private static BufferedReader br;
  private static PrintWriter out;

  private static String next() {
    while (st == null || !st.hasMoreElements()) {
      String s;
      try {
        s = br.readLine();
      } catch (IOException e) {
        return null;
      }
      st = new StringTokenizer(s);
    }
    return st.nextToken();
  }

  private static int nextInt() {
    return Integer.parseInt(next());
  }

  private static long nextLong() {
    return Long.parseLong(next());
  }

  public static void main(String[] args) {
    run();
  }
}