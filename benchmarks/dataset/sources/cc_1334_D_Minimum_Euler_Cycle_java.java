import java.util.*;
import java.io.*;
 
public class Main{
	static PrintWriter out=new PrintWriter(System.out);
	public static void main (String[] args) throws IOException{
		BufferedReader br=new BufferedReader(new InputStreamReader(System.in));
		
		String[] input=br.readLine().trim().split(" ");
		
		int numTestCases=Integer.parseInt(input[0]);
		while(numTestCases-->0){
		    input=br.readLine().trim().split(" ");
		    int n=Integer.parseInt(input[0]);
    		long l=Long.parseLong(input[1]);
    		long r=Long.parseLong(input[2]);
    		printSequence(n,l,r);
		}
        
		
		out.flush();
		out.close();
	}
	public static void printSequence(int n,long l,long r)
	{
	    ArrayList<Integer> ans=new ArrayList<>();
	    long total=1;
	    for(int i=1;i<n && l<=r;i++){
	        long len=n-i;
	        if(total+2*(len)<=l){
	            total+=2*len;
	            continue;
	        }
	        for(int j=i+1;j<=n;j++){
	            if(total==l && l<=r)
	            {
	                ans.add(i);
	                l++;
	            }
	            total++;
	            if(total==l && l<=r){
	                ans.add(j);
	                l++;
	            }
	            total++;
	        }
	    }
	    if(l<=r){
	        ans.add(1);
	    }
        for(int i=0;i<ans.size();i++){
            out.print(ans.get(i)+" ");
        }
	    out.println();
	}
}