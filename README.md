# lsedge

List edges in a distance table.

## Usage

Pass a distance table via stdin:

```sh
lsedge <<EOF
,A,B,C
A,0,3,0
B,3,0,4
C,0,4,0
EOF
```

The output will be:

```
from,to,weight
A,B,3
B,A,3
B,C,4
C,B,4
```
