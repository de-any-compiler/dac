/* dac --target c -O1 reconstruction
   input: tests/fixtures/hello-x86_64
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
    int8_t v5 = 0LL;
    int64_t v6 = arg0;
    int64_t v7 = arg1;
    int64_t v8 = arg2;
    int64_t v9 = arg3;
    int64_t v10 = arg4;
    int64_t v11 = arg5;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;

    v1 = (v0 - 8LL);
    v2 = (*((int64_t *)(16336LL)));
    v3 = v2;
    v4 = (v3 & v3);
    v5 = (v4 == 0LL);
    if (v5) {
    } else {
        v12 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v6, v7, v8, v9, v10, v11);
    }
    /* phi v13 <- (bb0: v3) (bb1: v12) */
    v14 = (v1 + 8LL);
    return v13;
}

/* dac-recovered function */
/* address: 0x1030 */
/* end: 0x1040 */
/* confidence: 0.85 (Derived) */
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
void fn_1030(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1040 */
/* end: 0x105e */
/* confidence: 1.00 (Observed) */
/* source_blocks: 2 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.28) */
/* args: rcx */
/* return_reg: rax */
/* stack_locals: 1 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
/* user_hint: id=1 rename=user_main return_override=true args_override=1 */
int32_t user_main(int32_t arg0) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    uint64_t n = 0LL;
    int32_t fd = 0LL;
    void * buf = ((void *)(0LL));
    int32_t v6 = arg0;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int32_t v10 = 0LL;
    int64_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int64_t v13 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    n = 6LL;
    fd = 1LL;
    buf = ((void *)(8196LL));
    v9 = ((long long (*)(long long, long long, long long, long long, long long, long long))fn_1030)(fd, ((int64_t)(buf)), n, v6, v7, v8);
    v10 = 42LL;
    v11 = (*((int64_t *)(((int64_t)(v1)))));
    v12 = ((void *)((((int64_t)(v1)) + 8LL)));
    v13 = v11;
    return v10;
}

/* dac-recovered function */
/* address: 0x1060 */
/* end: 0x1086 */
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
    void * v4 = ((void *)(0LL));
    int64_t v5 = 0LL;
    void * v6 = ((void *)(0LL));
    int64_t v7 = 0LL;
    void * v8 = ((void *)(0LL));
    void * v9 = ((void *)(0LL));
    void * v10 = ((void *)(0LL));
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = arg2;
    int64_t v14 = 0LL;
    int64_t v15 = arg0;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;

    v1 = (v0 ^ v0);
    v3 = v2;
    v5 = (*((int64_t *)(((int64_t)(v4)))));
    v6 = ((void *)((((int64_t)(v4)) + 8LL)));
    v7 = v5;
    v8 = ((void *)(((int64_t)(v6))));
    v9 = ((void *)((((int64_t)(v6)) & -16LL)));
    v10 = ((void *)((((int64_t)(v9)) - 8LL)));
    *((int64_t *)(((int64_t)(v10)))) = v11;
    v12 = (((int64_t)(v10)) - 8LL);
    *((int64_t *)(v12)) = v12;
    v14 = (v13 ^ v13);
    v16 = (v15 ^ v15);
    v17 = 4160LL;
    v18 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v17, v7, ((int64_t)(v8)), v16, v14, v3);
    (/* opaque: hlt */ 0);
    /* structurally unreachable: block 1 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1090 */
/* end: 0x10c0 */
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
    int8_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int8_t v6 = 0LL;
    int64_t v7 = 0LL;

    v0 = 16408LL;
    v2 = (v0 == v0);
    if (v2) {
L0:;
        /* phi v7 <- (bb0: v0) (bb1: v4) */
        return v7;
    } else {
        v3 = (*((int64_t *)(16328LL)));
        v4 = v3;
        v5 = (v4 & v4);
        v6 = (v5 == 0LL);
        if (v6) {
            goto L0;
        } else {
            /* structurally unreachable: block 2 */
            __builtin_unreachable();
        }
    }
}

/* dac-recovered function */
/* address: 0x10c0 */
/* end: 0x1100 */
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
    int8_t v11 = 0LL;
    int64_t v12 = 0LL;

    v0 = 16408LL;
    v2 = (v0 - v0);
    v3 = v2;
    v4 = (v2 >> 63LL);
    v5 = (v3 >> 3LL);
    v6 = (v4 + v5);
    v7 = (v6 >> 1LL);
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1100 */
/* end: 0x1150 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 3 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.70) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 1 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t __do_global_dtors_aux(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int8_t v0 = 0LL;
    int8_t v1 = 0LL;
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    void * v6 = ((void *)(0LL));
    int64_t v7 = arg0;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int64_t v10 = arg1;
    int64_t v11 = arg2;
    int64_t v12 = arg3;
    int64_t v13 = arg4;
    int64_t v14 = arg5;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    void * v19 = ((void *)(0LL));
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;

    v0 = (*((int8_t *)(16408LL)));
    v1 = (v0 != 0LL);
    if (v1) {
        return v21;
    } else {
        v3 = ((void *)((v2 - 8LL)));
        *((int64_t *)(((int64_t)(v3)))) = v4;
        v5 = (*((int64_t *)(16352LL)));
        v6 = ((void *)(((int64_t)(v3))));
        /* structurally unreachable: block 1 */
        __builtin_unreachable();
    }
}

/* dac-recovered function */
/* address: 0x1150 */
/* end: 0x1159 */
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
/* address: 0x115c */
/* end: 0x1169 */
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
