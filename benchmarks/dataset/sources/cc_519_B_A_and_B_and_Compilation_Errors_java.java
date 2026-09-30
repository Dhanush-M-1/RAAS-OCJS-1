
import java.util.Scanner;

/**
 *
 * @author Marco
 */
public class Main {

    /**
     * @param args the command line arguments
     */
    public static void main(String[] args) {
        // TODO code application logic here
        
        Scanner in = new Scanner(System.in);
        int cantidad = in.nextInt();
    long aux = 0, suma = 0;
        int num1, num2;
        for (int i = 0; i < cantidad; i++) {
            suma += in.nextInt();
            
        }
        for (int i = 0; i < cantidad - 1; i++) {
        aux += in.nextInt();
            
        }
        num1 = (int) (suma - aux);
        aux = 0;
        System.out.println(num1);
        for (int i = 0; i < cantidad - 2; i++) {
            aux += in.nextInt();
            
        }
        
        num2 = (int) (suma - aux - num1);
        System.out.println(num2);
        
    }
    
}
