import java.util.Scanner;
public class Main {
    public static void main (String [] args) {
        Scanner in = new Scanner(System.in);
        int r1 = in.nextInt();
        int r2 = in.nextInt();
        int c1 = in.nextInt();
        int c2 = in.nextInt();
        int d1 = in.nextInt();
        int d2 = in.nextInt();
        
        int a1=0, a2=0, a3=0, a4=0;
        boolean sw = true;
        for (int i=1; i<=9; i++) {
            a1 = i;
            a2 = r1 - a1;
            a3 = c1 - a1;
            a4 = d1 - a1;
            
            if( r2 == a3 + a4 && c2 == a2 + a4 && d2 == a3 + a2 ) {
                if( !(a2 < 1 || a3 < 1 || a4 < 1 || a2 > 9 || a3 > 9 || a4 > 9 || a1 == a2 || a1 == a3 || a1 == a4 || a2 == a3 || a2 == a4 || a3 == a4) ) {
                    System.out.println(a1 + " " + a2);
                    System.out.println(a3 + " " + a4);
                    sw = false;
                    break;
                }
            }
            
        }
        if (sw) {
            System.out.println("-1");
        }
    }
}