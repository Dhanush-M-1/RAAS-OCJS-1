import java.io.*;
import java.util.*;

public class Main
{
    public static void main(String args[])throws Exception
    {
        BufferedReader bu=new BufferedReader(new InputStreamReader(System.in));
        StringBuilder sb=new StringBuilder();
        String s[]=bu.readLine().split(" ");
        int n=Integer.parseInt(s[0]),k=Integer.parseInt(s[1]);
        ArrayList<Integer> ab=new ArrayList<>();
        ArrayList<Integer> a=new ArrayList<>();
        ArrayList<Integer> b=new ArrayList<>();
        int i,al=0,bo=0,x,y,z;
        for(i=0;i<n;i++)
        {
            s=bu.readLine().split(" ");
            x=Integer.parseInt(s[0]); y=Integer.parseInt(s[1]); z=Integer.parseInt(s[2]);
            if(y==1) al++;
            if(z==1) bo++;
            if(y==1 && z==1) ab.add(x);
            else if(y==1) a.add(x);
            else if(z==1) b.add(x);
        }
        if(al<k || bo<k) {System.out.print("-1"); return;}
        Collections.sort(ab); Collections.sort(a); Collections.sort(b);

        ArrayList<Integer> alb=new ArrayList<>();
        for(i=0;i<Math.min(a.size(),b.size());i++)
            alb.add(a.get(i)+b.get(i));
        int min=0,c=0;
        if(alb.size()==0)
        {
            for(i=0;i<k;i++)
                min+=ab.get(i);
            System.out.print(min);
            return;
        }
        if(ab.size()==0)
        {
            for(i=0;i<k;i++)
                min+=alb.get(i);
            System.out.print(min);
            return;
        }

        x=0; y=0;
        while(x<ab.size() && y<alb.size() && c<k)
        {
            if(ab.get(x)<alb.get(y)) {min+=ab.get(x); x++;}
            else {min+=alb.get(y); y++;}
            c++;
        }
        if(c==k) {System.out.print(min); return;}

        while(x<ab.size() && c<k)
        {
            min+=ab.get(x);
            x++;
            c++;
        }
        while(y<alb.size() && c<k)
        {
            min+=alb.get(y);
            y++;
            c++;
        }
        System.out.print(min);
    }
}