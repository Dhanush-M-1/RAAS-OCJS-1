import java.util.*;

public class Main {
  public static void main(String[] args) {
    Scanner s = new Scanner(System.in);
    int t = s.nextInt();
    while (t-- > 0) {
      int n = s.nextInt();
      int[] a = new int[n];
      for (int i = 0; i < n; ++i) {
        a[i] = s.nextInt();
      }
      if (a[n - 1] >= a[0] + a[1]) {
        System.out.println("1 2 " + n);
      } else {
        System.out.println(-1);
      }
    }
  }
}