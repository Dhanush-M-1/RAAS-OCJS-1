import java.util.Scanner;

public class Main {

	public static void main(String[] args) {
		// TODO Auto-generated method stub
		Scanner sc = new Scanner(System.in);
		int n = sc.nextInt();
		int a = sc.nextInt(); //0.5 liter
		int b = sc.nextInt(); //1
		int c = sc.nextInt(); //2
		
		int cnt = 0;
		for(int i=c;i>=0;i--){
			for(int j=b;j>=0;j--){
				int val = n-(i*2+j*1);
				if(val>=0 && val/0.5<=a) {
					cnt++;
				}
				/*
				for(int k=a;k>=0;k--){
					double sum = (i*2.0)+(j*1.0)+(k*0.5);
					//System.out.println(" 2리터 : "+i+", 1리터 : "+j+", 0.5리터 : "+k+" "+sum);
					if(sum == n) {
						cnt++;
					}
				}
				*/
			}
		}
		System.out.println(cnt);
	}

}
