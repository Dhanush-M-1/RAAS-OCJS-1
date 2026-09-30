import java.util.Scanner;

public class Main {

	/**
	 * @param args
	 */
	public static void main(String[] args) {
		// TODO Auto-generated method stub
		Scanner sc = new Scanner(System.in);
		int jugadores = sc.nextInt();
		String status = null;
		int mostrar = 0;
		int a = 0;
		int i = 0;
		int f = 0;
		status = sc.next();
		for (int l = 0; l < jugadores; l++) {
			if (status.substring(l, l + 1).equals("I")) {
				i++;
			} else if (status.substring(l, l + 1).equals("A")) {
				a++;
			}
			if (i > 1) {
				mostrar = 0;
			} else if (i == 1) {
				mostrar = 1;
			} else if (i == 0) {
				mostrar = a;
			}
		}
		System.out.println(mostrar);
	}
}