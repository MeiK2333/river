#include <cstdio>
#include <cstdlib>
#include <ctime>
using namespace std;

int t = 10;

double random(double start, double end);

int main(int argc, char const *argv[]) {
	printf("%d\n", t);
	double random(double, double);
	srand(unsigned(time(0)));
	while(t--) {
		int n = int(random(1, 100000));
		printf("%d\n", n);
		for(int i=0; i<n; ++i) {
			int a[31], cnt = 0;
			int num = int(random(0, 30));
			while(num>=1 && num<=30) {
				a[cnt++] = num;
				if(cnt >= 2) break;
				int interval = int(random(15, 30));
				num = int(random(num+1, num+interval));
			}
			printf("%d", cnt);
			for(int j=0; j<cnt; ++j) {
				printf(" %d", a[j]);
			}
			printf("\n");
		}
		int q = int(random(1, 10000));
		printf("%d\n", q);
		while(q--) {
			int l = int(random(1, n));
			int r = int(random(l, n));
			printf("%d %d\n", l, r);
		}
		if(t) printf("\n");
	}
	
	return 0;
}

double random(double start, double end) {
    return start+(end-start)*rand()/(RAND_MAX + 1.0);
}