import java.io.*;
import java.util.*;

public class Main {
    InputStream is;
    PrintWriter out;
    String INPUT = "";
    
    void solve(int TC) throws Exception {
    	int n = ni();
    	char[][] s = new char[n][n];
    	for(int i=0;i<n;i++) s[i] = ns(n);
    	//bfs..
    	int a1 = s[0][1]-'0';
    	int a2 = s[1][0]-'0';
    	int a3 = s[n-1][n-2]-'0';
    	int a4 = s[n-2][n-1]-'0';
    	if(a1==a2 && a3==a4 && a1!=a3) {
    		pn(0);
    		return;
    	}
    	if(a1!=a2 && a3!=a4) {
    		if(a1==a3) {
        		pn(2);
        		pn(2+" "+1);
        		pn(n+" "+(n-1));
        	}else {
        		pn(2);
            	pn("1 2");
            	pn(n+" "+(n-1));
        	}
    		return;
    	}
    	int one = a1+a2+a3+a4;
    	if(one==4) {
    		pn(2);
    		pn("1 2");
    		pn("2 1");
    		return;
    	}
    	if(one==0) {
    		pn(2);
    		pn("1 2");
    		pn("2 1");
    		return;
    	}
    	if(one==1) {
    		pn(1);
    		if(a1==1) {
    			pn("2 1");
    		}else if(a2==1) {
    			pn("1 2");
    		}else if(a3==1) {
    			pn((n-1)+" "+n);
    		}else{
    			pn(n+" "+(n-1));
    		}
    	}else{
    		pn(1);
    		if(a1==0) {
    			pn("2 1");
    		}else if(a2==0) {
    			pn("1 2");
    		}else if(a3==0) {
    			pn((n-1)+" "+n);
    		}else{
    			pn(n+" "+(n-1));
    		}
    	}
    }
    
    boolean TestCases = true;
    public static void main(String[] args) throws Exception { new Main().run(); }
    
    void hold(boolean b)throws Exception{if(!b)throw new Exception("Hold right there, Sparky!");}
    static void dbg(Object... o){System.err.println(Arrays.deepToString(o));}
    
    void run() throws Exception {
        is = INPUT.isEmpty() ? System.in : new ByteArrayInputStream(INPUT.getBytes());
        out = new PrintWriter(System.out);
        long s = System.currentTimeMillis();
        int T = TestCases ? ni() : 1;
        for(int t=1;t<=T;t++) solve(t);
        out.flush();
        if(!INPUT.isEmpty())tr(System.currentTimeMillis()-s+"ms");
    }
    
    void p(Object o) { out.print(o); }
    void pn(Object o) { out.println(o); }
    void pni(Object o) { out.println(o);out.flush(); }
    
    int ni() {
        int num = 0, b;
        boolean minus = false;
        while((b = readByte()) != -1 && !((b >= '0' && b <= '9') || b == '-'));
        if(b == '-') {
            minus = true;
            b = readByte();
        }
        while(true) {
            if(b >= '0' && b <= '9'){
                num = num * 10 + (b - '0');
            } else {
                return minus ? -num : num;
            }
            b = readByte();
        }
    }
    
    long nl() {
        long num = 0;
        int b;
        boolean minus = false;
        while((b = readByte()) != -1 && !((b >= '0' && b <= '9') || b == '-'));
        if(b == '-') {
            minus = true;
            b = readByte();
        }
        while(true) {
            if(b >= '0' && b <= '9') {
                num = num * 10 + (b - '0');
            } else {
                return minus ? -num : num;
            }
            b = readByte();
        }
    }
    
    double nd() { return Double.parseDouble(ns()); }
    char nc() { return (char)skip(); }
    
    int BUF_SIZE = 1024 * 8;
    byte[] inbuf = new byte[BUF_SIZE];
    int lenbuf = 0, ptrbuf = 0;
    
    int readByte() {
        if(lenbuf == -1)throw new InputMismatchException();
        if(ptrbuf >= lenbuf){
            ptrbuf = 0;
            try { lenbuf = is.read(inbuf); } catch (IOException e) { throw new InputMismatchException(); }
            if(lenbuf <= 0)return -1;
        } return inbuf[ptrbuf++];
    }
    
    boolean isSpaceChar(int c) { return !(c >= 33 && c <= 126); }
    int skip() { int b; while((b = readByte()) != -1 && isSpaceChar(b)); return b; }
    
    String ns() {
        int b = skip();
        StringBuilder sb = new StringBuilder();
        while(!(isSpaceChar(b))) {
            sb.appendCodePoint(b); b = readByte();
        } return sb.toString();
    }
    
    char[] ns(int n) {
        char[] buf = new char[n];
        int b = skip(), p = 0;
        while(p < n && !(isSpaceChar(b))){
            buf[p++] = (char)b;
            b = readByte();
        } return n == p ? buf : Arrays.copyOf(buf, p);
    }
    
    void tr(Object... o) { if(INPUT.length() > 0)System.out.println(Arrays.deepToString(o)); }
}   