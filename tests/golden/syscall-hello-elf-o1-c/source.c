/* dac --target c -O1 reconstruction
   input: tests/fixtures/syscall-hello-x86_64
   arch:  x86-64 */
#include <stdint.h>
#include <stddef.h>

/* dac-recovered function */
/* address: 0x1000 */
/* end: 0x101b */
/* confidence: 1.00 (Observed) */
/* source_blocks: 3 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.85) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t _init(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;
    int8_t v7 = 0LL;
    int64_t v8 = arg0;
    int64_t v9 = arg1;
    int64_t v10 = arg2;
    int64_t v11 = arg3;
    int64_t v12 = arg4;
    int64_t v13 = arg5;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;

    v1 = (v0 - 8LL);
    v3 = (v2 + 16336LL);
    v4 = (*((int64_t *)(v3)));
    v5 = v4;
    v6 = (v5 & v5);
    v7 = (v6 == 0LL);
    if (v7) {
    } else {
        v14 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v8, v9, v10, v11, v12, v13);
    }
    /* phi v15 <- (bb0: v5) (bb1: v14) */
    v16 = (v1 + 8LL);
    return v15;
}

/* dac-recovered function */
/* address: 0x1020 */
/* end: 0x1039 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 2 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64-syscall (score 0.75) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t main(void) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;

    v0 = 1LL;
    v2 = (v1 + 8196LL);
    v3 = v2;
    v4 = 18LL;
    v5 = v0;
    (/* opaque: syscall */ 0);
    v6 = (v0 ^ v0);
    return v6;
}

/* dac-recovered function */
/* address: 0x1040 */
/* end: 0x1066 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 2 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.47) */
/* args: rcx,rdx,r8 */
/* return_reg: none */
/* stack_locals: 1 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void _start(int64_t arg0, int64_t arg1, int64_t arg2) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = arg1;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = arg2;
    int64_t v14 = 0LL;
    int64_t v15 = arg0;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;

    v1 = (v0 ^ v0);
    v3 = v2;
    v5 = (*((int64_t *)(v4)));
    v6 = (v4 + 8LL);
    v7 = v5;
    v8 = v6;
    v9 = (v6 & -16LL);
    v10 = (v9 - 8LL);
    *((int64_t *)(v10)) = v11;
    v12 = (v10 - 8LL);
    *((int64_t *)(v12)) = v12;
    v14 = (v13 ^ v13);
    v16 = (v15 ^ v15);
    v18 = (v17 + 4128LL);
    v19 = v18;
    v20 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v19, v7, v8, v16, v14, v3);
    (/* opaque: hlt */ 0);
    /* structurally unreachable: block 1 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1070 */
/* end: 0x10a0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 4 */
/* goto_count: 1 */
/* label_count: 1 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.55) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t deregister_tm_clones(void) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int8_t v10 = 0LL;
    int64_t v11 = 0LL;

    v1 = (v0 + 16400LL);
    v2 = v1;
    v5 = (v2 == v2);
    if (v5) {
L0:;
        /* phi v11 <- (bb0: v2) (bb1: v8) */
        return v11;
    } else {
        v6 = (v0 + 16328LL);
        v7 = (*((int64_t *)(v6)));
        v8 = v7;
        v9 = (v8 & v8);
        v10 = (v9 == 0LL);
        if (v10) {
            goto L0;
        } else {
            /* structurally unreachable: block 2 */
            __builtin_unreachable();
        }
    }
}

/* dac-recovered function */
/* address: 0x10a0 */
/* end: 0x10e0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.55) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t register_tm_clones(void) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int8_t v15 = 0LL;
    int64_t v16 = 0LL;

    v1 = (v0 + 16400LL);
    v2 = v1;
    v5 = (v2 - v2);
    v6 = v5;
    v7 = (v5 >> 63LL);
    v8 = (v6 >> 3LL);
    v9 = (v7 + v8);
    v10 = (v9 >> 1LL);
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x10e0 */
/* end: 0x1130 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 3 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.70) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 1 */
/* struct_layouts: pointer=1 stack=0 */
/* switch_tables: 0 */
int64_t __do_global_dtors_aux(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int8_t v2 = 0LL;
    int8_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int64_t v10 = arg0;
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = arg1;
    int64_t v15 = arg2;
    int64_t v16 = arg3;
    int64_t v17 = arg4;
    int64_t v18 = arg5;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int64_t v26 = 0LL;

    v1 = (v0 + 16400LL);
    v2 = (*((int8_t *)(v1)));
    v3 = (v2 != 0LL);
    if (v3) {
        return v26;
    } else {
        v5 = (v4 - 8LL);
        *((int64_t *)(v5)) = v6;
        v7 = (v0 + 16352LL);
        v8 = (*((int64_t *)(v7)));
        v9 = v5;
        /* structurally unreachable: block 1 */
        __builtin_unreachable();
    }
}

/* dac-recovered function */
/* address: 0x1130 */
/* end: 0x1139 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void frame_dummy(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x113c */
/* end: 0x1149 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.40) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t _fini(void) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;

    v1 = (v0 - 8LL);
    v2 = (v1 + 8LL);
    return v3;
}
