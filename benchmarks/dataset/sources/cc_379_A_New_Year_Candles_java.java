
import java.util.Scanner;

public class Main {

	public static void main(String[] args) {
		Scanner in = new Scanner(System.in);

		int a = in.nextInt();
		int b = in.nextInt();

		int num_hours = a;

		while (true) {
			if (a < b) {
				break;
			}
			num_hours += a / b;
			a = a / b + a % b;
		}
		
		System.out.println(num_hours);
	}

}
