
import java.util.Scanner;

/*
 * To change this license header, choose License Headers in Project Properties.
 * To change this template file, choose Tools | Templates
 * and open the template in the editor.
 */
/**
 *
 * @author anhnth37
 */
public class Main {

    public static void main(String[] args) {
        Scanner input = new Scanner(System.in);
        int n = input.nextInt();
        String s = input.next();
        int maxPhoneNumber = n / 11;
        int countNumberEight = 0;
        for (int i = 0; i < n; i++) {
            if (s.charAt(i) == '8') {
                countNumberEight++;
            }
        }

        if (countNumberEight > maxPhoneNumber) {
            System.out.println(maxPhoneNumber);
        } else {
            System.out.println(countNumberEight);
        }
    }
}
