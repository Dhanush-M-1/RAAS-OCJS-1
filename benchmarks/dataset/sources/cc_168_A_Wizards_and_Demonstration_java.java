
import java.util.Scanner;


public class Main {

    public static void main(String[] args) {
        Scanner read = new Scanner(System.in);
        int n = read.nextInt();
        int x = read.nextInt();
        int y = read.nextInt();
        
        int minCount = (int) Math.ceil( (double)(n)*(double)(y)/100 );
        minCount-=x;
        if (minCount<0) minCount=0;
        System.out.print(minCount);
    }
}
