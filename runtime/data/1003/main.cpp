#include <cstdio>
#include <cstring>
using namespace std;

const int MAX = 100001;
int st[MAX*4];

void Initialize();
void Update(int node, int l, int r, int index, int value);
int Query(int node, int l, int r, int il, int ir);

int main(int argc, char const *argv[]) {
	int t, n, m, q, l, r;
	scanf("%d", &t);
	for(int i=1; i<=t; ++i) {
		printf("Case %d:\n", i);
		scanf("%d", &n);
		Initialize();
		for(int j=1; j<=n; ++j) {
			scanf("%d", &m);
			int binary = 0, idx;
			while(m--) {
				scanf("%d", &idx);
				binary |= 1<<(idx-1);
			}
			Update(1, 1, n, j, binary);
		}
		scanf("%d", &q);
		while(q--) {
			scanf("%d %d", &l, &r);
			int res = Query(1, 1, n, l, r);
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

void Initialize() {
	memset(st, 0, sizeof(st));
}

void Update(int node, int l, int r, int index, int value) {
	int mid = (l + r) >> 1;
	if(l == r) {
		st[node] |= value;
		return;
	}
	if(index <= mid)
		Update(node<<1, l, mid, index, value);
	else
		Update(node<<1|1, mid+1, r, index, value);
	st[node] = st[node<<1] | st[node<<1|1];
}

int Query(int node, int l, int r, int il, int ir) {
	int mid = (l + r) >> 1;
	if(l == il && r == ir)
		return st[node];
	if(ir <= mid)
		return Query(node<<1, l, mid, il, ir);
	else if(il > mid)
		return Query(node<<1|1, mid+1, r, il, ir);
	else
		return Query(node<<1, l, mid, il, mid)
			| Query(node<<1|1, mid+1, r, mid+1, ir);
}