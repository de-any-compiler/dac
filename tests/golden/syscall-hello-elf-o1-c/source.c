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
long _init(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    int64_t v2 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = arg0;
    int64_t v7 = arg1;
    int64_t v8 = arg2;
    int64_t v9 = arg3;
    int64_t v10 = arg4;
    int64_t v11 = arg5;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;

    v2 = (*((int64_t *)(16336LL)));
    v5 = (v2 == 0LL);
    if (v5) {
    } else {
        v12 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v6, v7, v8, v9, v10, v11);
    }
    /* phi v13 <- (bb0: v2) (bb1: v12) */
    return ((long)(v13));
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
int main(void) {
    (/* opaque: syscall */ 0);
    return ((int)(0LL));
}

/* dac-recovered function */
/* address: 0x1040 */
/* end: 0x1066 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 2 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.00) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 1 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void _start(void) {
    int64_t v0 = 0LL;
    int64_t v2 = 0LL;
    void * v4 = ((void *)(0LL));
    int64_t v5 = 0LL;
    void * v6 = ((void *)(0LL));
    void * v9 = ((void *)(0LL));
    void * v10 = ((void *)(0LL));
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v15 = 0LL;
    int64_t v18 = 0LL;

    v5 = (*((int64_t *)(((int64_t)(v4)))));
    v6 = ((void *)((((int64_t)(v4)) + 8LL)));
    v9 = ((void *)((((int64_t)(v6)) & -16LL)));
    v10 = ((void *)((((int64_t)(v9)) - 8LL)));
    *((int64_t *)(((int64_t)(v10)))) = v11;
    v12 = (((int64_t)(v10)) - 8LL);
    *((int64_t *)(v12)) = v12;
    v18 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(4128LL, v5, ((int64_t)(v6)), 0LL, 0LL, v2);
    (/* opaque: hlt */ 0);
    /* dac: structuring fallback */
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
long deregister_tm_clones(void) {
    int8_t v2 = 0LL;
    int64_t v3 = 0LL;
    int8_t v6 = 0LL;
    int64_t v7 = 0LL;

    v2 = (16400LL == 16400LL);
    if (v2) {
L0:;
        /* phi v7 <- (bb0: 16400) (bb1: v3) */
        return ((long)(v7));
    } else {
        v3 = (*((int64_t *)(16328LL)));
        v6 = (v3 == 0LL);
        if (v6) {
            goto L0;
        } else {
            /* dac: structuring fallback */
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
long register_tm_clones(void) {
    int64_t v8 = 0LL;
    int8_t v11 = 0LL;
    int64_t v12 = 0LL;

    /* dac: structuring fallback */
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
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
long __do_global_dtors_aux(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int8_t v0 = 0LL;
    int8_t v1 = 0LL;
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v7 = arg0;
    int64_t v8 = 0LL;
    int64_t v10 = arg1;
    int64_t v11 = arg2;
    int64_t v12 = arg3;
    int64_t v13 = arg4;
    int64_t v14 = arg5;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v21 = 0LL;

    v0 = (*((int8_t *)(16400LL)));
    v1 = (v0 != 0LL);
    if (v1) {
        return ((long)(v21));
    } else {
        v3 = ((void *)((v2 - 8LL)));
        *((int64_t *)(((int64_t)(v3)))) = v4;
        v5 = (*((int64_t *)(16352LL)));
        /* dac: structuring fallback */
    }
}

/* dac-recovered forwarding thunk */
/* address: 0x1130 */
/* end: 0x1139 */
/* confidence: 1.00 (Observed) */
/* tail-call: register_tm_clones (0x10a0) */
void frame_dummy(void) {
    register_tm_clones();
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
long _fini(void) {
    int64_t v0 = 0LL;
    int64_t v3 = 0LL;

    return ((long)(v3));
}
