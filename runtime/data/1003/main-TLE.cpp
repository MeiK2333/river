#include <cstdio>
using namespace std;

const int MAX = 100001;

int main(int argc, char const *argv[]) {
	int t, n, m, q, l, r, a[MAX];
	scanf("%d", &t);
	for(int i=1; i<=t; ++i) {
		printf("Case %d:\n", i);
		scanf("%d", &n);
		for(int j=1; j<=n; ++j) {
			scanf("%d", &m);
			int binary = 0, idx;
			while(m--) {
				scanf("%d", &idx);
				binary |= 1<<(idx-1);
			}
			a[j] = binary;
		}
		scanf("%d", &q);
		while(q--) {
			scanf("%d %d", &l, &r);
			int res = 0;
			for(int j=l; j<=r; ++j) {
				res |= a[j];
			}
			int digit = 1;
			bool is_first = true;
			while(res) {
				if(res & 1) {
					if(is_first) is_first = false;
					else printf(" ");
					printf("%d", digit);
				}
				res >>= 1;
				digit++;
			}
			if(digit == 1) printf("%%");
			printf("\n");
		}
	}
	
	return 0;
}