/* dac --target c -O1 reconstruction
   input: tests/fixtures/hello-x86_64.exe
   arch:  x86-64 */
#include <stdint.h>
#include <stddef.h>

/* dac-recovered struct */
/* base: v146 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_140001020_v146_t;

/* dac-recovered struct */
/* base: v152 */
/* total_size: 252 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_18[20];
    int16_t field_18;
    uint8_t __pad_1a_74[90];
    int32_t field_74;
    uint8_t __pad_78_84[12];
    int32_t field_84;
    uint8_t __pad_88_e8[96];
    int32_t field_e8;
    uint8_t __pad_ec_f8[12];
    int32_t field_f8;
} S_140001020_v152_t;

/* dac-recovered struct */
/* base: v17 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_20[32];
    int64_t field_20;
    uint8_t __pad_28_3c[20];
    int32_t field_3c;
    uint8_t __pad_40_4c[12];
    int32_t field_4c;
    uint8_t __pad_50_58[8];
    int64_t field_58;
} S_140001020_v17_t;

/* dac-recovered struct */
/* base: v6 */
/* total_size: 16 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_8[4];
    int64_t field_8;
} S_140001730_v6_t;

/* dac-recovered struct */
/* base: v5 */
/* total_size: 72 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_28[40];
    int64_t field_28;
    uint8_t __pad_30_58[40];
    int64_t field_58;
    int64_t field_60;
    int64_t field_68;
} S_140001830_v5_t;

/* dac-recovered struct */
/* base: v21 */
/* total_size: 16 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int64_t field_0;
    int64_t field_8;
} S_140001890_v21_t;

/* dac-recovered struct */
/* base: v48 */
/* total_size: 8 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_8[8];
    int32_t field_8;
    int32_t field_c;
} S_140001890_v48_t;

/* dac-recovered struct */
/* base: v57 */
/* total_size: 40 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_20[28];
    int64_t field_20;
} S_140001890_v57_t;

/* dac-recovered struct */
/* base: v7 */
/* total_size: 56 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_20[32];
    int64_t field_20;
    uint8_t __pad_28_38[16];
    int64_t field_38;
    uint8_t __pad_40_44[4];
    int32_t field_44;
    uint8_t __pad_48_50[8];
    int64_t field_50;
} S_140001890_v7_t;

/* dac-recovered struct */
/* base: v93 */
/* total_size: 16 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_8[8];
    int64_t field_8;
    int64_t field_10;
} S_140001890_v93_t;

/* dac-recovered struct */
/* base: v102 */
/* total_size: 12 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    int32_t field_4;
    int32_t field_8;
} S_140001a00_v102_t;

/* dac-recovered struct */
/* base: v217 */
/* total_size: 24 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_8[4];
    int64_t field_8;
    int64_t field_10;
} S_140001a00_v217_t;

/* dac-recovered struct */
/* base: v260 */
/* total_size: 8 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    int32_t field_4;
} S_140001a00_v260_t;

/* dac-recovered struct */
/* base: v27 */
/* total_size: 16 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int64_t field_0;
    int64_t field_8;
} S_140001a00_v27_t;

/* dac-recovered struct */
/* base: v68 */
/* total_size: 12 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    int32_t field_4;
    int32_t field_8;
} S_140001a00_v68_t;

/* dac-recovered struct */
/* base: v76 */
/* total_size: 8 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    int32_t field_4;
} S_140001a00_v76_t;

/* dac-recovered struct */
/* base: v1 */
/* total_size: 16 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_20[32];
    int32_t field_20;
    uint8_t __pad_24_28[4];
    int64_t field_28;
} S_140001d90_v1_t;

/* dac-recovered struct */
/* base: v2 */
/* total_size: 5 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int64_t field_0;
    int8_t field_4;
} S_140001de0_v2_t;

/* dac-recovered struct */
/* base: v6 */
/* total_size: 5 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    int8_t field_4;
} S_140001f80_v6_t;

/* dac-recovered struct */
/* base: v24 */
/* total_size: 24 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_8[4];
    int64_t field_8;
    int64_t field_10;
} S_140002140_v24_t;

/* dac-recovered struct */
/* base: v1 */
/* total_size: 40 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_28[40];
    int64_t field_28;
    uint8_t __pad_30_40[16];
    int32_t field_40;
    uint8_t __pad_44_48[4];
    int64_t field_48;
} S_1400021b0_v1_t;

/* dac-recovered struct */
/* base: v20 */
/* total_size: 16 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_8[4];
    int64_t field_8;
} S_1400021b0_v20_t;

/* dac-recovered struct */
/* base: v25 */
/* total_size: 24 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_10[12];
    int64_t field_10;
} S_140002240_v25_t;

/* dac-recovered struct */
/* base: v2 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_1400023f0_v2_t;

/* dac-recovered struct */
/* base: v8 */
/* total_size: 26 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_18[20];
    int16_t field_18;
} S_1400023f0_v8_t;

/* dac-recovered struct */
/* base: v24 */
/* total_size: 8 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_8[8];
    int32_t field_8;
    int32_t field_c;
} S_140002420_v24_t;

/* dac-recovered struct */
/* base: v4 */
/* total_size: 16 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_6[6];
    int16_t field_6;
    uint8_t __pad_8_14[12];
    int16_t field_14;
} S_140002420_v4_t;

/* dac-recovered struct */
/* base: v18 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_140002470_v18_t;

/* dac-recovered struct */
/* base: v24 */
/* total_size: 26 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_6[2];
    int16_t field_6;
    uint8_t __pad_8_14[12];
    int16_t field_14;
    uint8_t __pad_16_18[2];
    int16_t field_18;
} S_140002470_v24_t;

/* dac-recovered struct */
/* base: v1 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_140002510_v1_t;

/* dac-recovered struct */
/* base: v37 */
/* total_size: 8 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_8[8];
    int32_t field_8;
    int32_t field_c;
} S_140002510_v37_t;

/* dac-recovered struct */
/* base: v9 */
/* total_size: 26 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_6[2];
    int16_t field_6;
    uint8_t __pad_8_14[12];
    int16_t field_14;
    uint8_t __pad_16_18[2];
    int16_t field_18;
} S_140002510_v9_t;

/* dac-recovered struct */
/* base: v1 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_140002590_v1_t;

/* dac-recovered struct */
/* base: v9 */
/* total_size: 26 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_6[2];
    int16_t field_6;
    uint8_t __pad_8_18[16];
    int16_t field_18;
} S_140002590_v9_t;

/* dac-recovered struct */
/* base: v1 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_1400025d0_v1_t;

/* dac-recovered struct */
/* base: v9 */
/* total_size: 26 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_6[2];
    int16_t field_6;
    uint8_t __pad_8_14[12];
    int16_t field_14;
    uint8_t __pad_16_18[2];
    int16_t field_18;
} S_1400025d0_v9_t;

/* dac-recovered struct */
/* base: v1 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_140002650_v1_t;

/* dac-recovered struct */
/* base: v9 */
/* total_size: 26 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_18[20];
    int16_t field_18;
} S_140002650_v9_t;

/* dac-recovered struct */
/* base: v1 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_140002690_v1_t;

/* dac-recovered struct */
/* base: v37 */
/* total_size: 32 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_8[8];
    int32_t field_8;
    int32_t field_c;
    uint8_t __pad_10_24[20];
    int32_t field_24;
} S_140002690_v37_t;

/* dac-recovered struct */
/* base: v9 */
/* total_size: 26 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_6[2];
    int16_t field_6;
    uint8_t __pad_8_14[12];
    int16_t field_14;
    uint8_t __pad_16_18[2];
    int16_t field_18;
} S_140002690_v9_t;

/* dac-recovered struct */
/* base: v1 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_140002720_v1_t;

/* dac-recovered struct */
/* base: v40 */
/* total_size: 8 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_8[8];
    int32_t field_8;
    int32_t field_c;
} S_140002720_v40_t;

/* dac-recovered struct */
/* base: v52 */
/* total_size: 12 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_4[4];
    int32_t field_4;
    uint8_t __pad_8_c[4];
    int32_t field_c;
} S_140002720_v52_t;

/* dac-recovered struct */
/* base: v9 */
/* total_size: 148 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    uint8_t __pad_4_6[2];
    int16_t field_6;
    uint8_t __pad_8_14[12];
    int16_t field_14;
    uint8_t __pad_16_18[2];
    int16_t field_18;
    uint8_t __pad_1a_90[118];
    int32_t field_90;
} S_140002720_v9_t;

/* dac-recovered struct */
/* base: v3 */
/* total_size: 16 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int64_t field_0;
    int64_t field_8;
} S_1400027e0_v3_t;

/* dac-recovered struct */
/* base: v7 */
/* total_size: 24 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_20[32];
    int64_t field_20;
    uint8_t __pad_28_30[8];
    int64_t field_30;
} S_140002820_v7_t;

/* dac-recovered struct */
/* base: v7 */
/* total_size: 96 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_20[32];
    int64_t field_20;
    uint8_t __pad_28_38[16];
    int64_t field_38;
    int64_t field_40;
    uint8_t __pad_48_70[40];
    int64_t field_70;
    int64_t field_78;
} S_140002860_v7_t;

/* dac-recovered struct */
/* base: v9 */
/* total_size: 80 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_28[40];
    int64_t field_28;
    uint8_t __pad_30_70[64];
    int64_t field_70;
} S_140002900_v9_t;

/* dac-recovered struct */
/* base: v1 */
/* total_size: 48 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_20[32];
    int64_t field_20;
    uint8_t __pad_28_38[16];
    int32_t field_38;
    uint8_t __pad_3c_4c[16];
    int32_t field_4c;
} S_140002a60_v1_t;

/* dac-recovered function */
/* address: 0x140001000 */
/* end: 0x140001010 */
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
int64_t __mingw_invalidParameterHandler(void) {
    int64_t v0 = 0LL;

    return v0;
}

/* dac-recovered function */
/* address: 0x140001010 */
/* end: 0x140001020 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.48) */
/* args: rcx */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void safe_flush(int64_t arg0) {
    int64_t v0 = arg0;
    int64_t v1 = 0LL;

    v1 = (v0 ^ v0);
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140001020 */
/* end: 0x140001420 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 45 */
/* goto_count: 2 */
/* label_count: 2 */
/* irreducible: true */
/* convention: sysv-amd64 (score 0.85) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 11 */
/* struct_layouts: pointer=3 stack=1 */
/* switch_tables: 0 */
int64_t __tmainCRTStartup(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    int64_t v6 = 0LL;
    void * v7 = ((void *)(0LL));
    int64_t v8 = 0LL;
    void * v9 = ((void *)(0LL));
    int64_t v10 = 0LL;
    void * v11 = ((void *)(0LL));
    int64_t v12 = arg0;
    void * v13 = ((void *)(0LL));
    int64_t v14 = arg1;
    void * v15 = ((void *)(0LL));
    int64_t v16 = 0LL;
    S_140001020_v17_t * v17 = ((S_140001020_v17_t *)(0LL));
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int64_t v26 = 0LL;
    int64_t v27 = arg3;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int8_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = arg2;
    int64_t v34 = arg4;
    int64_t v35 = arg5;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int32_t v42 = 0LL;
    int32_t v43 = 0LL;
    int8_t v44 = 0LL;
    int32_t v45 = 0LL;
    int32_t v46 = 0LL;
    int32_t v47 = 0LL;
    int8_t v48 = 0LL;
    int64_t v49 = 0LL;
    int64_t v50 = 0LL;
    int32_t v51 = 0LL;
    int64_t v52 = 0LL;
    int64_t v53 = 0LL;
    int64_t v54 = 0LL;
    int64_t v55 = 0LL;
    int64_t v56 = 0LL;
    int8_t v57 = 0LL;
    int64_t v58 = 0LL;
    int64_t v59 = 0LL;
    int64_t v60 = 0LL;
    int64_t v61 = 0LL;
    int64_t v62 = 0LL;
    int8_t v63 = 0LL;
    int64_t v64 = 0LL;
    int64_t v65 = 0LL;
    int64_t v66 = 0LL;
    int64_t v67 = 0LL;
    int64_t v68 = 0LL;
    int64_t v69 = 0LL;
    int64_t v70 = 0LL;
    void * v71 = ((void *)(0LL));
    int64_t v72 = 0LL;
    int64_t v73 = 0LL;
    int32_t v74 = 0LL;
    int32_t v75 = 0LL;
    int64_t v76 = 0LL;
    int64_t v77 = 0LL;
    int32_t v78 = 0LL;
    int32_t v79 = 0LL;
    int32_t v80 = 0LL;
    int32_t v81 = 0LL;
    int8_t v82 = 0LL;
    int32_t v83 = 0LL;
    int32_t v84 = 0LL;
    int32_t v85 = 0LL;
    int8_t v86 = 0LL;
    int64_t v87 = 0LL;
    void * v88 = ((void *)(0LL));
    int64_t v89 = 0LL;
    void * v90 = ((void *)(0LL));
    int64_t v91 = 0LL;
    int64_t v92 = 0LL;
    void * v93 = ((void *)(0LL));
    int64_t v94 = 0LL;
    int64_t v95 = 0LL;
    void * v96 = ((void *)(0LL));
    int64_t v97 = 0LL;
    int64_t v98 = 0LL;
    void * v99 = ((void *)(0LL));
    int64_t v100 = 0LL;
    int64_t v101 = 0LL;
    void * v102 = ((void *)(0LL));
    int64_t v103 = 0LL;
    int64_t v104 = 0LL;
    void * v105 = ((void *)(0LL));
    int64_t v106 = 0LL;
    int64_t v107 = 0LL;
    void * v108 = ((void *)(0LL));
    int64_t v109 = 0LL;
    int64_t v110 = 0LL;
    void * v111 = ((void *)(0LL));
    int64_t v112 = 0LL;
    void * v113 = ((void *)(0LL));
    int64_t v114 = 0LL;
    void * v115 = ((void *)(0LL));
    int32_t v116 = 0LL;
    int32_t v117 = 0LL;
    int32_t v118 = 0LL;
    int64_t v119 = 0LL;
    int64_t v120 = 0LL;
    int64_t v121 = 0LL;
    int64_t v122 = 0LL;
    int64_t v123 = 0LL;
    int64_t v124 = 0LL;
    int64_t v125 = 0LL;
    int64_t v126 = 0LL;
    int64_t v127 = 0LL;
    int64_t v128 = 0LL;
    int64_t v129 = 0LL;
    int8_t v130 = 0LL;
    int64_t v131 = 0LL;
    int64_t v132 = 0LL;
    int64_t v133 = 0LL;
    int64_t v134 = 0LL;
    int64_t v135 = 0LL;
    int64_t v136 = 0LL;
    int64_t v137 = 0LL;
    int64_t v138 = 0LL;
    int64_t v139 = 0LL;
    int64_t v140 = 0LL;
    int64_t v141 = 0LL;
    int64_t v142 = 0LL;
    int64_t v143 = 0LL;
    int64_t v144 = 0LL;
    int64_t v145 = 0LL;
    S_140001020_v146_t * v146 = ((S_140001020_v146_t *)(0LL));
    int16_t v147 = 0LL;
    int8_t v148 = 0LL;
    int64_t v149 = 0LL;
    int32_t v150 = 0LL;
    int32_t v151 = 0LL;
    S_140001020_v152_t * v152 = ((S_140001020_v152_t *)(0LL));
    int32_t v153 = 0LL;
    int8_t v154 = 0LL;
    void * v155 = ((void *)(0LL));
    int16_t v156 = 0LL;
    int16_t v157 = 0LL;
    int8_t v158 = 0LL;
    int8_t v159 = 0LL;
    void * v160 = ((void *)(0LL));
    int32_t v161 = 0LL;
    int8_t v162 = 0LL;
    void * v163 = ((void *)(0LL));
    int32_t v164 = 0LL;
    int32_t v165 = 0LL;
    int64_t v166 = 0LL;
    void * v167 = ((void *)(0LL));
    int32_t v168 = 0LL;
    int8_t v169 = 0LL;
    void * v170 = ((void *)(0LL));
    int32_t v171 = 0LL;
    int32_t v172 = 0LL;
    int64_t v173 = 0LL;
    int32_t v174 = 0LL;
    int64_t v175 = 0LL;
    int64_t v176 = 0LL;
    int64_t v177 = 0LL;
    int64_t v178 = 0LL;
    int32_t v179 = 0LL;
    int32_t v180 = 0LL;
    int32_t v181 = 0LL;
    int8_t v182 = 0LL;
    int64_t v183 = 0LL;
    int64_t v184 = 0LL;
    int64_t n = 0LL;
    int64_t v186 = 0LL;
    int64_t v187 = 0LL;
    int64_t v188 = 0LL;
    int64_t v189 = 0LL;
    int64_t v190 = 0LL;
    void * v191 = ((void *)(0LL));
    int64_t v192 = 0LL;
    int64_t v193 = 0LL;
    int32_t v194 = 0LL;
    int32_t v195 = 0LL;
    void * v196 = ((void *)(0LL));
    int64_t v197 = 0LL;
    int64_t v198 = 0LL;
    int32_t v199 = 0LL;
    int32_t v200 = 0LL;
    int64_t v201 = 0LL;
    int64_t v202 = 0LL;
    int64_t v203 = 0LL;
    int32_t v204 = 0LL;
    int8_t v205 = 0LL;
    int64_t v206 = 0LL;
    int64_t v207 = 0LL;
    int32_t v208 = 0LL;
    int8_t v209 = 0LL;
    int64_t v210 = 0LL;
    int64_t v211 = 0LL;
    int64_t v212 = 0LL;
    int64_t v213 = 0LL;
    int64_t v214 = 0LL;
    int64_t v215 = 0LL;
    int8_t v216 = 0LL;
    int64_t v217 = 0LL;
    int64_t v218 = 0LL;
    int64_t v219 = 0LL;
    int64_t v220 = 0LL;
    int64_t v221 = 0LL;
    int32_t v222 = 0LL;
    int32_t v223 = 0LL;
    void * v224 = ((void *)(0LL));
    int64_t v225 = 0LL;
    int64_t v226 = 0LL;
    int32_t v227 = 0LL;
    int32_t v228 = 0LL;
    int64_t v229 = 0LL;
    int64_t v230 = 0LL;
    void * v231 = ((void *)(0LL));
    int64_t v232 = 0LL;
    int32_t v233 = 0LL;
    int32_t v234 = 0LL;
    int32_t v235 = 0LL;
    int32_t v236 = 0LL;
    int32_t v237 = 0LL;
    int32_t v238 = 0LL;
    void * v239 = ((void *)(0LL));
    int64_t v240 = 0LL;
    void * v241 = ((void *)(0LL));
    int8_t v242 = 0LL;
    int32_t v243 = 0LL;
    int8_t v244 = 0LL;
    int64_t v245 = 0LL;
    int64_t v246 = 0LL;
    int64_t v247 = 0LL;
    int64_t s = 0LL;
    void * src = ((void *)(0LL));
    uint64_t v250 = 0LL;
    int64_t v251 = 0LL;
    int64_t v252 = 0LL;
    void * v253 = ((void *)(0LL));
    void * v254 = ((void *)(0LL));
    int64_t v255 = 0LL;
    int64_t v256 = 0LL;
    uint64_t v257 = 0LL;
    uint64_t v258 = 0LL;
    int64_t dst = 0LL;
    int64_t v260 = 0LL;
    int64_t v261 = 0LL;
    int64_t v262 = 0LL;
    void * v263 = ((void *)(0LL));
    void * v264 = ((void *)(0LL));
    int64_t v265 = 0LL;
    int8_t v266 = 0LL;
    int64_t v267 = 0LL;
    void * v268 = ((void *)(0LL));
    void * v269 = ((void *)(0LL));
    int64_t v270 = 0LL;
    uint64_t n_1 = 0LL;
    int64_t v272 = 0LL;
    int64_t v273 = 0LL;
    void * v274 = ((void *)(0LL));
    int8_t v275 = 0LL;
    void * v276 = ((void *)(0LL));
    void * v277 = ((void *)(0LL));
    void * v278 = ((void *)(0LL));
    int64_t v279 = 0LL;
    int64_t v280 = 0LL;
    int64_t v281 = 0LL;
    int64_t v282 = 0LL;
    void * v283 = ((void *)(0LL));
    int64_t v284 = 0LL;
    int64_t v285 = 0LL;
    int64_t v286 = 0LL;
    int64_t v287 = 0LL;
    int64_t v288 = 0LL;
    int64_t v289 = 0LL;
    int64_t v290 = 0LL;
    int64_t v291 = 0LL;
    int64_t v292 = 0LL;
    int64_t v293 = 0LL;
    int64_t v294 = 0LL;
    int64_t v295 = 0LL;
    int64_t v296 = 0LL;
    int64_t v297 = 0LL;
    int64_t v298 = 0LL;
    int64_t v299 = 0LL;
    int64_t v300 = 0LL;
    int64_t v301 = 0LL;
    int64_t v302 = 0LL;
    int64_t v303 = 0LL;
    int64_t v304 = 0LL;
    int64_t v305 = 0LL;
    int64_t v306 = 0LL;
    int64_t v307 = 0LL;
    int64_t v308 = 0LL;
    int64_t v309 = 0LL;
    int64_t v310 = 0LL;
    int64_t v311 = 0LL;
    int64_t v312 = 0LL;
    int64_t v313 = 0LL;
    int64_t v314 = 0LL;
    int64_t v315 = 0LL;
    int64_t v316 = 0LL;
    int64_t status = 0LL;
    int64_t v318 = 0LL;
    int32_t v319 = 0LL;
    int64_t v320 = 0LL;
    int64_t v321 = 0LL;
    int64_t v322 = 0LL;
    int32_t v323 = 0LL;
    int64_t v324 = 0LL;
    int64_t v325 = 0LL;
    int64_t v326 = 0LL;
    int64_t v327 = 0LL;
    int64_t v328 = 0LL;
    int64_t v329 = 0LL;
    int64_t v330 = 0LL;
    int64_t v331 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 8LL)));
    *((int64_t *)(((int64_t)(v5)))) = v6;
    v7 = ((void *)((((int64_t)(v5)) - 8LL)));
    *((int64_t *)(((int64_t)(v7)))) = v8;
    v9 = ((void *)((((int64_t)(v7)) - 8LL)));
    *((int64_t *)(((int64_t)(v9)))) = v10;
    v11 = ((void *)((((int64_t)(v9)) - 8LL)));
    *((int64_t *)(((int64_t)(v11)))) = v12;
    v13 = ((void *)((((int64_t)(v11)) - 8LL)));
    *((int64_t *)(((int64_t)(v13)))) = v14;
    v15 = ((void *)((((int64_t)(v13)) - 8LL)));
    *((int64_t *)(((int64_t)(v15)))) = v16;
    v17 = ((S_140001020_v17_t *)((((int64_t)(v15)) - 88LL)));
    v18 = (*((int64_t *)(48LL)));
    v19 = v18;
    v20 = (v19 + 8LL);
    v21 = (*((int64_t *)(v20)));
    v22 = v21;
    v23 = (*((int64_t *)(5368726704LL)));
    v24 = v23;
    v25 = (*((int64_t *)(5368746720LL)));
    v26 = v25;
    while (1) {
        /* phi v28 <- (bb0: v19) (bb3: v36) */
        /* phi v29 <- (bb0: v27) (bb3: v32) */
        v30 = (v28 ^ v28);
        (/* opaque: cmpxchg */ 0);
        /* structurally unreachable: block 4 */
        __builtin_unreachable();
    }
    /* phi v39 <- (bb5: v38) (bb18: v37) */
    v40 = (*((int64_t *)(5368726720LL)));
    v41 = v40;
    v42 = (*((int32_t *)(v41)));
    v43 = v42;
    v44 = (v43 == 1LL);
    if (v44) {
L0:;
        /* phi v325 <- (bb6: v26) (bb79: status) */
        /* phi v326 <- (bb6: v22) (bb79: v318) */
        /* phi v327 <- (bb6: v33) (bb79: v320) */
        /* phi v328 <- (bb6: v34) (bb79: v321) */
        /* phi v329 <- (bb6: v35) (bb79: v322) */
        v330 = 31LL;
        v331 = ((long long (*)(long long, long long, long long, long long, long long, long long))_amsg_exit)(v325, v326, v327, v330, v328, v329);
        /* structurally unreachable: block 81 */
        __builtin_unreachable();
    } else {
        v45 = (*((int32_t *)(v41)));
        v46 = v45;
        v47 = (v46 & v46);
        v48 = (v47 == 0LL);
        if (v48) {
            v119 = 2LL;
            *((int32_t *)(v41)) = 1LL;
            v120 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v26, v22, v33, v119, v34, v35);
            v121 = (v35 ^ v35);
            v122 = 4LL;
            v123 = (v33 ^ v33);
            v124 = v120;
            v125 = ((long long (*)(long long, long long, long long, long long, long long, long long))setvbuf)(v26, v22, v123, v124, v122, v121);
            v126 = 5368713232LL;
            v127 = ((long long (*)(long long, long long, long long, long long, long long, long long))_crt_atexit)(v26, v22, v123, v126, v122, v121);
            v128 = v127;
            v129 = (v127 & v127);
            v130 = (v129 != 0LL);
            if (v130) {
                v309 = ((long long (*)(long long, long long, long long, long long, long long, long long))abort)(v26, v128, v123, v126, v122, v121);
                /* phi v310 <- (bb48: n) (bb77: v26) */
                /* phi v311 <- (bb48: v186) (bb77: v128) */
                /* phi v312 <- (bb48: v211) (bb77: v123) */
                /* phi v313 <- (bb48: v189) (bb77: v122) */
                /* phi v314 <- (bb48: v190) (bb77: v121) */
                v315 = 10LL;
                v316 = ((long long (*)(long long, long long, long long, long long, long long, long long))_amsg_exit)(v310, v311, v312, v315, v313, v314);
L1:;
                /* phi status <- (bb14: v49) (bb78: v310) */
                /* phi v318 <- (bb14: v50) (bb78: v311) */
                /* phi v319 <- (bb14: v78) (bb78: v316) */
                /* phi v320 <- (bb14: v77) (bb78: v312) */
                /* phi v321 <- (bb14: v73) (bb78: v313) */
                /* phi v322 <- (bb14: v55) (bb78: v314) */
                v323 = v319;
                v324 = ((long long (*)(long long, long long, long long, long long, long long, long long))exit)(status, v318, v320, v323, v321, v322);
                goto L0;
            } else {
                v131 = ((long long (*)(long long, long long, long long, long long, long long, long long))_pei386_runtime_relocator)(v26, v128, v123, v126, v122, v121);
                v132 = 5368717184LL;
                v133 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v26, v128, v123, v132, v122, v121);
                v134 = (*((int64_t *)(5368726688LL)));
                v135 = v134;
                v136 = 5368713216LL;
                *((int64_t *)(v135)) = v133;
                v137 = ((long long (*)(long long, long long, long long, long long, long long, long long))_set_invalid_parameter_handler)(v26, v128, v135, v136, v122, v121);
                v138 = ((long long (*)(long long, long long, long long, long long, long long, long long))_fpreset)(v26, v128, v135, v136, v122, v121);
                v139 = (*((int64_t *)(5368726640LL)));
                v140 = v139;
                *((int32_t *)(v140)) = 1LL;
                v141 = (*((int64_t *)(5368726656LL)));
                v142 = v141;
                *((int32_t *)(v142)) = 1LL;
                v143 = (*((int64_t *)(5368726672LL)));
                v144 = v143;
                *((int32_t *)(v144)) = 1LL;
                v145 = (*((int64_t *)(5368726528LL)));
                v146 = ((S_140001020_v146_t *)(v145));
                v147 = (*((int16_t *)(((int64_t)(v146)))));
                v148 = (v147 != 23117LL);
                if (v148) {
                } else {
                    v149 = (((int64_t)(v146)) + 60LL);
                    v150 = v146->field_3c;
                    v151 = v150;
                    v152 = ((S_140001020_v152_t *)((((int64_t)(v146)) + v151)));
                    v153 = (*((int32_t *)(((int64_t)(v152)))));
                    v154 = (v153 != 17744LL);
                    if (v154) {
                    } else {
                        v155 = ((void *)((((int64_t)(v152)) + 24LL)));
                        v156 = v152->field_18;
                        v157 = v156;
                        v158 = (v157 == 267LL);
                        if (v158) {
                            v167 = ((void *)((((int64_t)(v152)) + 116LL)));
                            v168 = v152->field_74;
                            v169 = (v168 <= 14LL);
                            if (v169) {
                            } else {
                                v170 = ((void *)((((int64_t)(v152)) + 232LL)));
                                v171 = v152->field_e8;
                                v172 = v171;
                                v173 = (v128 ^ v128);
                                (/* opaque: setne */ 0);
                            }
                        } else {
                            v159 = (v157 != 523LL);
                            if (v159) {
                            } else {
                                v160 = ((void *)((((int64_t)(v152)) + 132LL)));
                                v161 = v152->field_84;
                                v162 = (v161 <= 14LL);
                                if (v162) {
                                } else {
                                    v163 = ((void *)((((int64_t)(v152)) + 248LL)));
                                    v164 = v152->field_f8;
                                    v165 = v164;
                                    v166 = (v128 ^ v128);
                                    (/* opaque: setne */ 0);
                                }
                            }
                        }
                    }
                }
                /* phi v174 <- (bb33: v128) (bb34: v128) (bb36: v128) (bb37: v128) (bb38: v166) (bb75: v128) (bb76: v173) */
                /* phi v175 <- (bb33: v135) (bb34: v151) (bb36: v157) (bb37: v157) (bb38: v157) (bb75: v157) (bb76: v157) */
                /* phi v176 <- (bb33: v121) (bb34: v121) (bb36: v121) (bb37: v121) (bb38: v165) (bb75: v121) (bb76: v121) */
                v177 = (*((int64_t *)(5368726624LL)));
                v178 = v177;
                *((int32_t *)(5368741896LL)) = v174;
                v179 = (*((int32_t *)(v178)));
                v180 = v179;
                v181 = (v180 & v180);
                v182 = (v181 != 0LL);
                if (v182) {
                    /* phi v302 <- (bb39: v26) (bb63: v295) */
                    /* phi v303 <- (bb39: v174) (bb63: v296) */
                    /* phi v304 <- (bb39: v175) (bb63: v297) */
                    /* phi v305 <- (bb39: v180) (bb63: v298) */
                    /* phi v306 <- (bb39: v176) (bb63: v299) */
                    v307 = 2LL;
                    v308 = ((long long (*)(long long, long long, long long, long long, long long, long long))__set_app_type)(v302, v303, v304, v307, v305, v306);
                } else {
                    v183 = 1LL;
                    v184 = ((long long (*)(long long, long long, long long, long long, long long, long long))__set_app_type)(v26, v174, v175, v183, v180, v176);
                }
                /* phi n <- (bb40: v26) (bb65: v302) */
                /* phi v186 <- (bb40: v174) (bb65: v303) */
                /* phi v187 <- (bb40: v183) (bb65: v307) */
                /* phi v188 <- (bb40: v175) (bb65: v304) */
                /* phi v189 <- (bb40: v180) (bb65: v305) */
                /* phi v190 <- (bb40: v176) (bb65: v306) */
                v191 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__p__fmode)(n, v186, v188, v187, v189, v190)));
                v192 = (*((int64_t *)(5368726832LL)));
                v193 = v192;
                v194 = (*((int32_t *)(v193)));
                v195 = v194;
                *((int32_t *)(((int64_t)(v191)))) = v195;
                v196 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__p__commode)(n, v186, v195, v187, v189, v190)));
                v197 = (*((int64_t *)(5368726800LL)));
                v198 = v197;
                v199 = (*((int32_t *)(v198)));
                v200 = v199;
                *((int32_t *)(((int64_t)(v196)))) = v200;
                v201 = ((long long (*)(long long, long long, long long, long long, long long, long long))_setargv)(n, v186, v200, v187, v189, v190);
                /* structurally unreachable: block 44 */
                __builtin_unreachable();
            }
        } else {
            *((int32_t *)(5368741892LL)) = 1LL;
            /* phi v49 <- (bb8: v26) (bb70: v281) */
            /* phi v50 <- (bb8: v22) (bb70: v282) */
            /* phi v51 <- (bb8: v46) (bb70: v290) */
            /* phi v52 <- (bb8: v29) (bb70: v288) */
            /* phi v53 <- (bb8: v33) (bb70: v286) */
            /* phi v54 <- (bb8: v34) (bb70: v284) */
            /* phi v55 <- (bb8: v35) (bb70: v228) */
            v56 = (v39 & v39);
            v57 = (v56 == 0LL);
            if (v57) {
                v118 = (v51 ^ v51);
                (/* opaque: xchg */ 0);
            }
            v58 = (*((int64_t *)(5368726576LL)));
            v59 = v58;
            v60 = (*((int64_t *)(v59)));
            v61 = v60;
            v62 = (v61 & v61);
            v63 = (v62 == 0LL);
            if (v63) {
            } else {
                v64 = (v54 ^ v54);
                v65 = 2LL;
                v66 = (v52 ^ v52);
                v67 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v49, v50, v65, v66, v64, v55);
            }
            /* phi v68 <- (bb10: v52) (bb11: v66) */
            /* phi v69 <- (bb10: v53) (bb11: v65) */
            /* phi v70 <- (bb10: v54) (bb11: v64) */
            v71 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__p___initenv)(v49, v50, v69, v68, v70, v55)));
            v72 = (*((int64_t *)(5368741904LL)));
            v73 = v72;
            v74 = (*((int32_t *)(5368741920LL)));
            v75 = v74;
            *((int64_t *)(((int64_t)(v71)))) = v73;
            v76 = (*((int64_t *)(5368741912LL)));
            v77 = v76;
            v78 = ((long long (*)(long long, long long, long long, long long, long long, long long))main)(v49, v50, v77, v75, v73, v55);
            v79 = (*((int32_t *)(5368741896LL)));
            v80 = v79;
            v81 = (v80 & v80);
            v82 = (v81 == 0LL);
            if (v82) {
                goto L1;
            } else {
                v83 = (*((int32_t *)(5368741892LL)));
                v84 = v83;
                v85 = (v84 & v84);
                v86 = (v85 == 0LL);
                if (v86) {
                    v113 = ((void *)((((int64_t)(v17)) + 60LL)));
                    v17->field_3c = v78;
                    v114 = ((long long (*)(long long, long long, long long, long long, long long, long long))_cexit)(v49, v50, v84, v80, v73, v55);
                    v115 = ((void *)((((int64_t)(v17)) + 60LL)));
                    v116 = v17->field_3c;
                    v117 = v116;
                }
                /* phi v87 <- (bb15: v78) (bb21: v117) */
                v88 = ((void *)((((int64_t)(v17)) + 88LL)));
                v89 = v17->field_58;
                v90 = ((void *)((((int64_t)(v88)) + 8LL)));
                v91 = v89;
                v92 = (*((int64_t *)(((int64_t)(v90)))));
                v93 = ((void *)((((int64_t)(v90)) + 8LL)));
                v94 = v92;
                v95 = (*((int64_t *)(((int64_t)(v93)))));
                v96 = ((void *)((((int64_t)(v93)) + 8LL)));
                v97 = v95;
                v98 = (*((int64_t *)(((int64_t)(v96)))));
                v99 = ((void *)((((int64_t)(v96)) + 8LL)));
                v100 = v98;
                v101 = (*((int64_t *)(((int64_t)(v99)))));
                v102 = ((void *)((((int64_t)(v99)) + 8LL)));
                v103 = v101;
                v104 = (*((int64_t *)(((int64_t)(v102)))));
                v105 = ((void *)((((int64_t)(v102)) + 8LL)));
                v106 = v104;
                v107 = (*((int64_t *)(((int64_t)(v105)))));
                v108 = ((void *)((((int64_t)(v105)) + 8LL)));
                v109 = v107;
                v110 = (*((int64_t *)(((int64_t)(v108)))));
                v111 = ((void *)((((int64_t)(v108)) + 8LL)));
                v112 = v110;
                return v87;
            }
        }
    }
}

/* dac-recovered function */
/* address: 0x140001420 */
/* end: 0x140001440 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 2 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.85) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t WinMainCRTStartup(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = arg0;
    int64_t v5 = arg1;
    int64_t v6 = arg2;
    int64_t v7 = arg3;
    int64_t v8 = arg4;
    int64_t v9 = arg5;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;

    v1 = (v0 - 40LL);
    v2 = (*((int64_t *)(5368726624LL)));
    v3 = v2;
    *((int32_t *)(v3)) = 1LL;
    v10 = ((long long (*)(long long, long long, long long, long long, long long, long long))__tmainCRTStartup)(v4, v5, v6, v7, v8, v9);
    v11 = (v1 + 40LL);
    return v10;
}

/* dac-recovered function */
/* address: 0x140001440 */
/* end: 0x140001460 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 2 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.85) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t mainCRTStartup(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = arg0;
    int64_t v5 = arg1;
    int64_t v6 = arg2;
    int64_t v7 = arg3;
    int64_t v8 = arg4;
    int64_t v9 = arg5;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;

    v1 = (v0 - 40LL);
    v2 = (*((int64_t *)(5368726624LL)));
    v3 = v2;
    *((int32_t *)(v3)) = 0LL;
    v10 = ((long long (*)(long long, long long, long long, long long, long long, long long))__tmainCRTStartup)(v4, v5, v6, v7, v8, v9);
    v11 = (v1 + 40LL);
    return v10;
}

/* dac-recovered function */
/* address: 0x140001460 */
/* end: 0x140001490 */
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
void atexit(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140001490 */
/* end: 0x140001540 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 9 */
/* goto_count: 1 */
/* label_count: 1 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.35) */
/* args: rdi,rsi,rdx */
/* return_reg: none */
/* stack_locals: 4 */
/* struct_layouts: pointer=0 stack=1 */
/* switch_tables: 0 */
void __gcc_register_frame(int64_t arg0, int64_t arg1, int64_t arg2) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    void * v6 = ((void *)(0LL));
    void * v7 = ((void *)(0LL));
    int64_t str_libgcc_s_dw2_1_dll = 0LL;
    int64_t v9 = arg0;
    int64_t v10 = arg1;
    int64_t v11 = arg2;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int8_t v17 = 0LL;
    int64_t str_libgcc_s_dw2_1_dll_1 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t str_register_frame_info = 0LL;
    int64_t v23 = 0LL;
    void * v24 = ((void *)(0LL));
    int64_t v25 = 0LL;
    int64_t str_deregister_frame_info = 0LL;
    int64_t v27 = 0LL;
    void * v28 = ((void *)(0LL));
    int64_t v29 = 0LL;
    void * v30 = ((void *)(0LL));
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int8_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    void * v41 = ((void *)(0LL));
    int64_t v42 = 0LL;
    void * v43 = ((void *)(0LL));
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    void * v46 = ((void *)(0LL));
    int64_t v47 = 0LL;
    int64_t v48 = 0LL;
    int64_t v49 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 56LL)));
    v6 = ((void *)((((int64_t)(v5)) + 48LL)));
    v7 = ((void *)(((int64_t)(v6))));
    str_libgcc_s_dw2_1_dll = 5368725504LL;
    v14 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v9, v10, v11, str_libgcc_s_dw2_1_dll, v12, v13);
    v15 = v14;
    v16 = (v14 & v14);
    v17 = (v16 == 0LL);
    if (v17) {
        v48 = 5368714368LL;
        v49 = 5368714352LL;
        *((int64_t *)(5368721408LL)) = v48;
L0:;
        /* phi v35 <- (bb5: v32) (bb8: v49) */
        /* phi v36 <- (bb5: v21) (bb8: v13) */
        v37 = 5368741984LL;
        v38 = 5368729600LL;
        v39 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v9, v10, v37, v38, v35, v36);
    } else {
        str_libgcc_s_dw2_1_dll_1 = 5368725504LL;
        v19 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v9, v10, v11, str_libgcc_s_dw2_1_dll_1, v12, v13);
        v20 = (*((int64_t *)(5368746672LL)));
        v21 = v20;
        str_register_frame_info = 5368725523LL;
        v23 = v15;
        *((int64_t *)(5368741952LL)) = v19;
        v24 = ((void *)((((int64_t)(v7)) + -16LL)));
        *((int64_t *)(((int64_t)(v24)))) = v21;
        v25 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v9, v10, str_register_frame_info, v23, v12, v21);
        str_deregister_frame_info = 5368725545LL;
        v27 = v15;
        v28 = ((void *)((((int64_t)(v7)) + -8LL)));
        *((int64_t *)(((int64_t)(v28)))) = v25;
        v29 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v9, v10, str_deregister_frame_info, v27, v12, v21);
        v30 = ((void *)((((int64_t)(v7)) + -8LL)));
        v31 = (*((int64_t *)(((int64_t)(v30)))));
        v32 = v31;
        *((int64_t *)(5368721408LL)) = v29;
        v33 = (v32 & v32);
        v34 = (v33 == 0LL);
        if (v34) {
        } else {
            goto L0;
        }
    }
    v40 = 5368714560LL;
    v41 = ((void *)((((int64_t)(v5)) + 56LL)));
    v42 = (*((int64_t *)(((int64_t)(v41)))));
    v43 = ((void *)((((int64_t)(v41)) + 8LL)));
    v44 = v42;
    v45 = (*((int64_t *)(((int64_t)(v43)))));
    v46 = ((void *)((((int64_t)(v43)) + 8LL)));
    v47 = v45;
    /* structurally unreachable: block 7 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140001540 */
/* end: 0x140001580 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 5 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.50) */
/* args: rdi,rsi,rdx */
/* return_reg: rax */
/* stack_locals: 1 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t __gcc_deregister_frame(int64_t arg0, int64_t arg1, int64_t arg2) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    void * v4 = ((void *)(0LL));
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;
    int64_t v7 = 0LL;
    int8_t v8 = 0LL;
    int64_t v9 = 0LL;
    int64_t v10 = arg0;
    int64_t v11 = arg1;
    int64_t v12 = arg2;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int8_t v20 = 0LL;
    void * v21 = ((void *)(0LL));
    int64_t v22 = 0LL;
    void * v23 = ((void *)(0LL));
    int64_t v24 = 0LL;
    void * v25 = ((void *)(0LL));
    int64_t v26 = 0LL;
    void * v27 = ((void *)(0LL));
    int64_t v28 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)(((int64_t)(v1))));
    v4 = ((void *)((((int64_t)(v1)) - 32LL)));
    v5 = (*((int64_t *)(5368721408LL)));
    v6 = v5;
    v7 = (v6 & v6);
    v8 = (v7 == 0LL);
    if (v8) {
    } else {
        v9 = 5368729600LL;
        v15 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v10, v11, v12, v9, v13, v14);
    }
    /* phi v16 <- (bb0: v6) (bb1: v15) */
    v17 = (*((int64_t *)(5368741952LL)));
    v18 = v17;
    v19 = (v18 & v18);
    v20 = (v19 == 0LL);
    if (v20) {
        v25 = ((void *)((((int64_t)(v4)) + 32LL)));
        v26 = (*((int64_t *)(((int64_t)(v25)))));
        v27 = ((void *)((((int64_t)(v25)) + 8LL)));
        v28 = v26;
        return v16;
    } else {
        v21 = ((void *)((((int64_t)(v4)) + 32LL)));
        v22 = (*((int64_t *)(((int64_t)(v21)))));
        v23 = ((void *)((((int64_t)(v21)) + 8LL)));
        v24 = v22;
        /* structurally unreachable: block 3 */
        __builtin_unreachable();
    }
}

/* dac-recovered function */
/* address: 0x140001580 */
/* end: 0x1400015d0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 5 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.85) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t __do_global_dtors(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;
    int8_t v7 = 0LL;
    int64_t v8 = arg2;
    int64_t v9 = 0LL;
    int64_t v10 = arg0;
    int64_t v11 = arg1;
    int64_t v12 = arg3;
    int64_t v13 = arg4;
    int64_t v14 = arg5;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int8_t v24 = 0LL;
    int64_t v25 = 0LL;
    int64_t v26 = 0LL;

    v1 = (v0 - 40LL);
    v2 = (*((int64_t *)(5368721424LL)));
    v3 = v2;
    v4 = (*((int64_t *)(v3)));
    v5 = v4;
    v6 = (v5 & v5);
    v7 = (v6 == 0LL);
    if (v7) {
    } else {
        while (1) {
            /* phi v9 <- (bb1: v8) (bb3: v19) */
            v15 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v10, v11, v9, v12, v13, v14);
            v16 = (*((int64_t *)(5368721424LL)));
            v17 = v16;
            v18 = (v17 + 8LL);
            v19 = v18;
            v21 = (*((int64_t *)(v18)));
            v22 = v21;
            *((int64_t *)(5368721424LL)) = v19;
            v23 = (v22 & v22);
            v24 = (v23 != 0LL);
            if (v24) {
                continue;
            }
        }
    }
    /* phi v25 <- (bb0: v5) (bb3: v22) */
    v26 = (v1 + 40LL);
    return v25;
}

/* dac-recovered function */
/* address: 0x1400015d0 */
/* end: 0x140001650 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 9 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.30) */
/* args: rdi,rsi */
/* return_reg: none */
/* stack_locals: 2 */
/* struct_layouts: pointer=0 stack=1 */
/* switch_tables: 0 */
void __do_global_ctors(int64_t arg0, int64_t arg1) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = arg1;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    int64_t v6 = 0LL;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int64_t v10 = 0LL;
    int8_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    int8_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = arg0;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int8_t v32 = 0LL;
    int64_t v33 = 0LL;
    void * v34 = ((void *)(0LL));
    int64_t v35 = 0LL;
    void * v36 = ((void *)(0LL));
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    void * v39 = ((void *)(0LL));
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    int64_t v47 = 0LL;
    int64_t v48 = 0LL;
    int64_t v49 = 0LL;
    int8_t v50 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 40LL)));
    v6 = (*((int64_t *)(5368726512LL)));
    v7 = v6;
    v8 = (*((int64_t *)(v7)));
    v9 = v8;
    v10 = v9;
    v11 = (v9 == -1LL);
    if (v11) {
        v41 = (v9 ^ v9);
        while (1) {
            /* phi v42 <- (bb7: v41) (bb8: v46) */
            v43 = (v42 + 1LL);
            v44 = v43;
            v45 = v42;
            v46 = v44;
            v47 = (v44 * 8LL);
            v48 = (v7 + v47);
            v49 = (*((int64_t *)(v48)));
            v50 = (v49 != 0LL);
            if (v50) {
                continue;
            } else {
                break;
            }
        }
    }
    /* phi v13 <- (bb0: v10) (bb9: v45) */
    /* phi v14 <- (bb0: v12) (bb9: v44) */
    v15 = (v13 & v13);
    v16 = (v15 == 0LL);
    if (v16) {
    } else {
        v17 = v13;
        v18 = (v13 - 1LL);
        v19 = (v17 * 8LL);
        v20 = (v7 + v19);
        v21 = v20;
        v22 = (v17 - v18);
        v23 = (v7 + -8LL);
        v24 = (v22 * 8LL);
        v25 = (v23 + v24);
        v26 = v25;
        while (1) {
            /* phi v27 <- (bb2: v21) (bb4: v31) */
            v30 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v28, v26, v7, v18, v14, v29);
            v31 = (v27 - 8LL);
            v32 = (v31 != v26);
            if (v32) {
                continue;
            }
        }
    }
    v33 = 5368714624LL;
    v34 = ((void *)((((int64_t)(v5)) + 40LL)));
    v35 = (*((int64_t *)(((int64_t)(v34)))));
    v36 = ((void *)((((int64_t)(v34)) + 8LL)));
    v37 = v35;
    v38 = (*((int64_t *)(((int64_t)(v36)))));
    v39 = ((void *)((((int64_t)(v36)) + 8LL)));
    v40 = v38;
    /* structurally unreachable: block 5 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140001650 */
/* end: 0x140001670 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 3 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.55) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t __main(void) {
    int32_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int8_t v3 = 0LL;

    v0 = (*((int32_t *)(5368742048LL)));
    v1 = v0;
    v2 = (v1 & v1);
    v3 = (v2 == 0LL);
    if (v3) {
        *((int32_t *)(5368742048LL)) = 1LL;
        /* structurally unreachable: block 3 */
        __builtin_unreachable();
    } else {
        return v1;
    }
}

/* dac-recovered function */
/* address: 0x140001670 */
/* end: 0x140001680 */
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
int64_t _setargv(void) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;

    v1 = (v0 ^ v0);
    return v1;
}

/* dac-recovered function */
/* address: 0x140001680 */
/* end: 0x1400016a0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 4 */
/* goto_count: 1 */
/* label_count: 1 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.30) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t __dyn_tls_dtor(void) {
    int64_t v0 = 0LL;
    int8_t v1 = 0LL;
    int64_t v2 = 0LL;
    int8_t v3 = 0LL;
    int64_t v4 = 0LL;

    v1 = (v0 == 3LL);
    if (v1) {
L0:;
        /* structurally unreachable: block 4 */
        __builtin_unreachable();
    } else {
        v2 = (v0 & v0);
        v3 = (v2 == 0LL);
        if (v3) {
            goto L0;
        } else {
            return v4;
        }
    }
}

/* dac-recovered function */
/* address: 0x1400016a0 */
/* end: 0x140001720 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 12 */
/* goto_count: 1 */
/* label_count: 1 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.85) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 2 */
/* struct_layouts: pointer=0 stack=1 */
/* switch_tables: 0 */
int64_t __dyn_tls_init(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = arg1;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    int64_t v6 = 0LL;
    int64_t v7 = 0LL;
    int32_t v8 = 0LL;
    int8_t v9 = 0LL;
    int64_t v10 = arg2;
    int8_t v11 = 0LL;
    int8_t v12 = 0LL;
    void * v13 = ((void *)(0LL));
    int64_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    void * v18 = ((void *)(0LL));
    int64_t v19 = 0LL;
    void * v20 = ((void *)(0LL));
    int64_t v21 = 0LL;
    void * v22 = ((void *)(0LL));
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    void * v25 = ((void *)(0LL));
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int8_t v29 = 0LL;
    void * v30 = ((void *)(0LL));
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int8_t v34 = 0LL;
    int64_t v35 = arg0;
    int64_t v36 = arg3;
    int64_t v37 = arg4;
    int64_t v38 = arg5;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    void * v41 = ((void *)(0LL));
    int8_t v42 = 0LL;
    void * v43 = ((void *)(0LL));
    int64_t v44 = 0LL;
    void * v45 = ((void *)(0LL));
    int64_t v46 = 0LL;
    int64_t v47 = 0LL;
    void * v48 = ((void *)(0LL));
    int64_t v49 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 40LL)));
    v6 = (*((int64_t *)(5368726480LL)));
    v7 = v6;
    v8 = (*((int32_t *)(v7)));
    v9 = (v8 == 2LL);
    if (v9) {
    } else {
        *((int32_t *)(v7)) = 2LL;
    }
    v11 = (v10 == 2LL);
    if (v11) {
        v27 = 5368726976LL;
        v29 = (v27 == v27);
        if (v29) {
L0:;
            v20 = ((void *)((((int64_t)(v5)) + 40LL)));
            v21 = (*((int64_t *)(((int64_t)(v20)))));
            v22 = ((void *)((((int64_t)(v20)) + 8LL)));
            v23 = v21;
            v24 = (*((int64_t *)(((int64_t)(v22)))));
            v25 = ((void *)((((int64_t)(v22)) + 8LL)));
            v26 = v24;
            return v7;
        } else {
            while (1) {
                /* phi v30 <- (bb7: v27) (bb10: v41) */
                v31 = (*((int64_t *)(((int64_t)(v30)))));
                v32 = v31;
                v33 = (v32 & v32);
                v34 = (v33 == 0LL);
                if (v34) {
                } else {
                    v39 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v35, v27, v10, v36, v37, v38);
                }
                /* phi v40 <- (bb8: v32) (bb9: v39) */
                v41 = ((void *)((((int64_t)(v30)) + 8LL)));
                v42 = (((int64_t)(v41)) != v27);
                if (v42) {
                    continue;
                } else {
                    break;
                }
            }
            v43 = ((void *)((((int64_t)(v5)) + 40LL)));
            v44 = (*((int64_t *)(((int64_t)(v43)))));
            v45 = ((void *)((((int64_t)(v43)) + 8LL)));
            v46 = v44;
            v47 = (*((int64_t *)(((int64_t)(v45)))));
            v48 = ((void *)((((int64_t)(v45)) + 8LL)));
            v49 = v47;
            return v40;
        }
    } else {
        v12 = (v10 == 1LL);
        if (v12) {
            v13 = ((void *)((((int64_t)(v5)) + 40LL)));
            v14 = (*((int64_t *)(((int64_t)(v13)))));
            v15 = ((void *)((((int64_t)(v13)) + 8LL)));
            v16 = v14;
            v17 = (*((int64_t *)(((int64_t)(v15)))));
            v18 = ((void *)((((int64_t)(v15)) + 8LL)));
            v19 = v17;
            /* structurally unreachable: block 13 */
            __builtin_unreachable();
        } else {
            goto L0;
        }
    }
}

/* dac-recovered function */
/* address: 0x140001720 */
/* end: 0x140001730 */
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
int64_t __tlregdtor(void) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;

    v1 = (v0 ^ v0);
    return v1;
}

/* dac-recovered function */
/* address: 0x140001730 */
/* end: 0x140001830 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 6 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.85) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 2 */
/* struct_layouts: pointer=1 stack=1 */
/* switch_tables: 1 */
int64_t _matherr(int64_t arg0, int64_t arg1, int64_t arg2, S_140001730_v6_t * arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = arg1;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    S_140001730_v6_t * v6 = arg3;
    int32_t v7 = 0LL;
    int8_t v8 = 0LL;
    int32_t v9 = 0LL;
    int32_t v10 = 0LL;
    int64_t v11 = 0LL;
    int32_t v12 = 0LL;
    int64_t v13 = 0LL;
    int32_t v14 = 0LL;
    int32_t v15 = 0LL;
    int32_t v16 = 0LL;
    int64_t str_unknown_error = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = arg0;
    int64_t v23 = arg2;
    int64_t v24 = arg4;
    int64_t v25 = arg5;
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    void * v33 = ((void *)(0LL));
    int64_t v34 = 0LL;
    void * v35 = ((void *)(0LL));
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    void * v38 = ((void *)(0LL));
    int64_t v39 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 120LL)));
    (/* opaque: movaps */ 0);
    (/* opaque: movaps */ 0);
    (/* opaque: movaps */ 0);
    v7 = (*((int32_t *)(((int64_t)(v6)))));
    v8 = (v7 > 6LL);
    if (v8) {
        str_unknown_error = 5368725926LL;
        (/* opaque: movsd */ 0);
        (/* opaque: movsd */ 0);
        (/* opaque: movsd */ 0);
        v18 = (((int64_t)(v6)) + 8LL);
        v19 = v6->field_8;
        v20 = v19;
        v21 = 2LL;
        v26 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v22, v20, v23, v21, v24, v25);
        (/* opaque: movsd */ 0);
        v27 = str_unknown_error;
        v28 = 5368725944LL;
        (/* opaque: movsd */ 0);
        v29 = v20;
        v30 = v26;
        (/* opaque: movsd */ 0);
        v31 = ((long long (*)(long long, long long, long long, long long, long long, long long))fprintf)(v22, v20, v28, v30, v27, v29);
        (/* opaque: movaps */ 0);
        (/* opaque: movaps */ 0);
        v32 = (v31 ^ v31);
        (/* opaque: movaps */ 0);
        v33 = ((void *)((((int64_t)(v5)) + 120LL)));
        v34 = (*((int64_t *)(((int64_t)(v33)))));
        v35 = ((void *)((((int64_t)(v33)) + 8LL)));
        v36 = v34;
        v37 = (*((int64_t *)(((int64_t)(v35)))));
        v38 = ((void *)((((int64_t)(v35)) + 8LL)));
        v39 = v37;
        return v32;
    } else {
        v9 = (*((int32_t *)(((int64_t)(v6)))));
        v10 = v9;
        v11 = 5368725988LL;
        v12 = (v10 * 4LL);
        v13 = (v11 + v12);
        v14 = (*((int32_t *)(v13)));
        v15 = v14;
        v16 = (v15 + v11);
        /* recovered switch table at block 1 (arm resolution pending) */
        switch (v10) {
            default: {
                /* structurally unreachable: block 1 */
                __builtin_unreachable();
            }
        }
    }
}

/* dac-recovered function */
/* address: 0x140001830 */
/* end: 0x140001890 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 6 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.80) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: none */
/* stack_locals: 6 */
/* struct_layouts: pointer=1 stack=1 */
/* switch_tables: 0 */
void __report_error(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    S_140001830_v5_t * v5 = ((S_140001830_v5_t *)(0LL));
    int64_t v6 = arg0;
    int64_t v7 = 0LL;
    void * v8 = ((void *)(0LL));
    int64_t v9 = 0LL;
    int64_t v10 = 0LL;
    void * v11 = ((void *)(0LL));
    int64_t v12 = arg2;
    void * v13 = ((void *)(0LL));
    int64_t v14 = arg3;
    int64_t v15 = 0LL;
    int64_t v16 = arg1;
    void * v17 = ((void *)(0LL));
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    void * v23 = ((void *)(0LL));
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((S_140001830_v5_t *)((((int64_t)(v3)) - 56LL)));
    v7 = v6;
    v8 = ((void *)((((int64_t)(v5)) + 88LL)));
    v9 = ((int64_t)(v8));
    v10 = 2LL;
    v11 = ((void *)((((int64_t)(v5)) + 96LL)));
    v5->field_60 = v12;
    v13 = ((void *)((((int64_t)(v5)) + 104LL)));
    v5->field_68 = v14;
    v5->field_58 = v16;
    v17 = ((void *)((((int64_t)(v5)) + 40LL)));
    v5->field_28 = v9;
    v19 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v18, v2, v16, v10, v12, v14);
    v20 = 5368726016LL;
    v21 = v19;
    v22 = ((long long (*)(long long, long long, long long, long long, long long, long long))fprintf)(v18, v2, v20, v21, v12, v14);
    v23 = ((void *)((((int64_t)(v5)) + 40LL)));
    v24 = v5->field_28;
    v25 = v24;
    v26 = 2LL;
    v27 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v18, v25, v20, v26, v12, v14);
    v28 = v7;
    v29 = v25;
    v30 = v27;
    v31 = ((long long (*)(long long, long long, long long, long long, long long, long long))vfprintf)(v18, v25, v28, v30, v29, v14);
    v32 = ((long long (*)(long long, long long, long long, long long, long long, long long))abort)(v18, v25, v28, v30, v29, v14);
    /* structurally unreachable: block 5 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140001890 */
/* end: 0x140001a00 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 16 */
/* goto_count: 3 */
/* label_count: 3 */
/* irreducible: true */
/* convention: sysv-amd64 (score 0.85) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 6 */
/* struct_layouts: pointer=5 stack=1 */
/* switch_tables: 0 */
int64_t mark_section_writable(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = arg0;
    void * v3 = ((void *)(0LL));
    int64_t v4 = arg1;
    void * v5 = ((void *)(0LL));
    int64_t v6 = 0LL;
    S_140001890_v7_t * v7 = ((S_140001890_v7_t *)(0LL));
    int32_t v8 = 0LL;
    int32_t v9 = 0LL;
    int64_t v10 = arg3;
    int64_t v11 = 0LL;
    int32_t v12 = 0LL;
    int8_t v13 = 0LL;
    int64_t v14 = arg5;
    int64_t v15 = arg4;
    int64_t v16 = arg2;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    S_140001890_v21_t * v21 = ((S_140001890_v21_t *)(0LL));
    int64_t i = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int8_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int32_t v31 = 0LL;
    int32_t v32 = 0LL;
    int64_t v33 = 0LL;
    int8_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int8_t v39 = 0LL;
    int64_t v40 = 0LL;
    int32_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    int64_t v47 = 0LL;
    S_140001890_v48_t * v48 = ((S_140001890_v48_t *)(0LL));
    int64_t v49 = 0LL;
    int8_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int32_t v53 = 0LL;
    int32_t v54 = 0LL;
    int32_t v55 = 0LL;
    int32_t v56 = 0LL;
    S_140001890_v57_t * v57 = ((S_140001890_v57_t *)(0LL));
    void * v58 = ((void *)(0LL));
    int64_t v59 = 0LL;
    int64_t v60 = 0LL;
    int32_t v61 = 0LL;
    int32_t v62 = 0LL;
    int64_t v63 = 0LL;
    int32_t v64 = 0LL;
    int64_t v65 = 0LL;
    int64_t v66 = 0LL;
    int64_t v67 = 0LL;
    void * v68 = ((void *)(0LL));
    void * v69 = ((void *)(0LL));
    int64_t v70 = 0LL;
    void * v71 = ((void *)(0LL));
    int64_t v72 = 0LL;
    int64_t v73 = 0LL;
    int8_t v74 = 0LL;
    void * v75 = ((void *)(0LL));
    int32_t v76 = 0LL;
    int32_t v77 = 0LL;
    int32_t v78 = 0LL;
    int32_t v79 = 0LL;
    int32_t v80 = 0LL;
    int32_t v81 = 0LL;
    int32_t v82 = 0LL;
    int32_t v83 = 0LL;
    void * v84 = ((void *)(0LL));
    int64_t v85 = 0LL;
    int64_t v86 = 0LL;
    void * v87 = ((void *)(0LL));
    int64_t v88 = 0LL;
    int64_t v89 = 0LL;
    int64_t v90 = 0LL;
    int64_t v91 = 0LL;
    int64_t v92 = 0LL;
    S_140001890_v93_t * v93 = ((S_140001890_v93_t *)(0LL));
    void * v94 = ((void *)(0LL));
    int64_t v95 = 0LL;
    void * v96 = ((void *)(0LL));
    int64_t v97 = 0LL;
    int64_t v98 = 0LL;
    int8_t v99 = 0LL;
    int64_t v100 = 0LL;
    int64_t str_virtualprotect_failed_wi = 0LL;
    int64_t v102 = 0LL;
    int64_t v103 = 0LL;
    int32_t v104 = 0LL;
    int32_t v105 = 0LL;
    int32_t v106 = 0LL;
    int64_t v107 = 0LL;
    int64_t v108 = 0LL;
    int64_t v109 = 0LL;
    int32_t v110 = 0LL;
    int32_t v111 = 0LL;
    int64_t str_virtualquery_failed_for = 0LL;
    int64_t v113 = 0LL;
    void * v114 = ((void *)(0LL));
    int64_t v115 = 0LL;
    int64_t v116 = 0LL;
    int64_t v117 = 0LL;
    int64_t v118 = 0LL;
    int64_t v119 = 0LL;
    int64_t v120 = 0LL;
    int64_t str_address_p_has_no_image_s = 0LL;
    int64_t v122 = 0LL;
    int64_t v123 = 0LL;
    void * v124 = ((void *)(0LL));
    int64_t v125 = 0LL;
    void * v126 = ((void *)(0LL));
    int64_t v127 = 0LL;
    int64_t v128 = 0LL;
    void * v129 = ((void *)(0LL));
    int64_t v130 = 0LL;
    int64_t v131 = 0LL;
    void * v132 = ((void *)(0LL));
    int64_t v133 = 0LL;
    int64_t v134 = 0LL;
    int32_t v135 = 0LL;
    int64_t v136 = 0LL;
    int64_t v137 = 0LL;
    int64_t v138 = 0LL;
    int64_t v139 = 0LL;
    int32_t v140 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 8LL)));
    *((int64_t *)(((int64_t)(v5)))) = v6;
    v7 = ((S_140001890_v7_t *)((((int64_t)(v5)) - 80LL)));
    v8 = (*((int32_t *)(5368742148LL)));
    v9 = v8;
    v11 = v10;
    v12 = (v9 & v9);
    v13 = (v12 <= 0LL);
    if (v13) {
        /* phi v134 <- (bb0: v2) (bb19: v48) */
        /* phi v135 <- (bb0: v9) (bb19: v41) */
        /* phi v136 <- (bb0: v11) (bb19: v93) */
        /* phi v137 <- (bb0: v14) (bb19: v95) */
        /* phi v138 <- (bb0: v15) (bb19: v90) */
        /* phi v139 <- (bb0: v16) (bb19: v102) */
        v140 = (v135 ^ v135);
L2:;
        /* phi v40 <- (bb4: v2) (bb20: v134) */
        /* phi v41 <- (bb4: v9) (bb20: v140) */
        /* phi v42 <- (bb4: v11) (bb20: v136) */
        /* phi v43 <- (bb4: v37) (bb20: v137) */
        /* phi v44 <- (bb4: v35) (bb20: v138) */
        /* phi v45 <- (bb4: v36) (bb20: v139) */
        v46 = v42;
        v47 = ((long long (*)(long long, long long, long long, long long, long long, long long))__mingw_GetSectionForAddress)(v40, v41, v45, v46, v44, v43);
        v48 = ((S_140001890_v48_t *)(v47));
        v49 = (v47 & v47);
        v50 = (v49 == 0LL);
        if (v50) {
L0:;
            /* phi v118 <- (bb6: v42) (bb21: v56) */
            /* phi v119 <- (bb6: v44) (bb21: v116) */
            v120 = v118;
            str_address_p_has_no_image_s = 5368726048LL;
            v122 = ((long long (*)(long long, long long, long long, long long, long long, long long))__report_error)(((int64_t)(v48)), v41, v120, str_address_p_has_no_image_s, v119, v43);
            /* structurally unreachable: block 23 */
            __builtin_unreachable();
        } else {
            v51 = (*((int64_t *)(5368742152LL)));
            v52 = v51;
            v53 = (v41 * 4LL);
            v54 = (v41 + v53);
            v55 = v54;
            v56 = (v55 << 3LL);
            v57 = ((S_140001890_v57_t *)((v52 + v56)));
            v58 = ((void *)((((int64_t)(v57)) + 32LL)));
            v57->field_20 = ((int64_t)(v48));
            *((int32_t *)(((int64_t)(v57)))) = 0LL;
            v59 = ((long long (*)(long long, long long, long long, long long, long long, long long))_GetPEImageBase)(((int64_t)(v48)), v41, v45, v46, v44, v43);
            v60 = (((int64_t)(v48)) + 12LL);
            v61 = v48->field_c;
            v62 = v61;
            v63 = 48LL;
            v64 = (v59 + v62);
            v65 = v64;
            v66 = (*((int64_t *)(5368742152LL)));
            v67 = v66;
            v68 = ((void *)((((int64_t)(v7)) + 32LL)));
            v69 = ((void *)(((int64_t)(v68))));
            v70 = (v67 + 24LL);
            v71 = ((void *)((v70 + v56)));
            *((int64_t *)(((int64_t)(v71)))) = v65;
            v72 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(((int64_t)(v48)), v41, ((int64_t)(v69)), v65, v63, v43);
            v73 = (v72 & v72);
            v74 = (v73 == 0LL);
            if (v74) {
                v107 = (*((int64_t *)(5368742152LL)));
                v108 = v107;
                v109 = (((int64_t)(v48)) + 8LL);
                v110 = v48->field_8;
                v111 = v110;
                str_virtualquery_failed_for = 5368726080LL;
                v113 = (v108 + 24LL);
                v114 = ((void *)((v113 + v56)));
                v115 = (*((int64_t *)(((int64_t)(v114)))));
                v116 = v115;
                v117 = ((long long (*)(long long, long long, long long, long long, long long, long long))__report_error)(((int64_t)(v48)), v41, v111, str_virtualquery_failed_for, v116, v43);
                goto L0;
            } else {
                v75 = ((void *)((((int64_t)(v7)) + 68LL)));
                v76 = v7->field_44;
                v77 = v76;
                v78 = (v77 + -4LL);
                v79 = v78;
                v80 = (v79 & -5LL);
                /* structurally unreachable: block 10 */
                __builtin_unreachable();
            }
        }
    } else {
        v17 = (*((int64_t *)(5368742152LL)));
        v18 = v17;
        v19 = (v14 ^ v14);
        v20 = (v18 + 24LL);
        while (1) {
            /* phi v21 <- (bb1: v20) (bb4: v38) */
            /* phi i <- (bb1: v19) (bb4: v37) */
            /* phi v23 <- (bb1: v16) (bb4: v36) */
            v24 = (*((int64_t *)(((int64_t)(v21)))));
            v25 = v24;
            v26 = (v11 < v25);
            if (v26) {
L1:;
                /* phi v35 <- (bb2: v25) (bb3: v33) */
                /* phi v36 <- (bb2: v23) (bb3: v32) */
                v37 = (i + 1LL);
                v38 = (((int64_t)(v21)) + 40LL);
                v39 = (v37 != v9);
                if (v39) {
                    continue;
                } else {
                    break;
                }
            } else {
                v27 = (((int64_t)(v21)) + 8LL);
                v28 = v21->field_8;
                v29 = v28;
                v30 = (v29 + 8LL);
                v31 = (*((int32_t *)(v30)));
                v32 = v31;
                v33 = (v25 + v32);
                v34 = (v11 < v33);
                if (v34) {
                    /* phi v123 <- (bb3: v21) (bb12: v104) */
                    v124 = ((void *)((((int64_t)(v7)) + 80LL)));
                    v125 = v7->field_50;
                    v126 = ((void *)((((int64_t)(v124)) + 8LL)));
                    v127 = v125;
                    v128 = (*((int64_t *)(((int64_t)(v126)))));
                    v129 = ((void *)((((int64_t)(v126)) + 8LL)));
                    v130 = v128;
                    v131 = (*((int64_t *)(((int64_t)(v129)))));
                    v132 = ((void *)((((int64_t)(v129)) + 8LL)));
                    v133 = v131;
                    return v123;
                } else {
                    goto L1;
                }
            }
        }
        goto L2;
    }
}

/* dac-recovered function */
/* address: 0x140001a00 */
/* end: 0x140001d90 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 33 */
/* goto_count: 9 */
/* label_count: 5 */
/* irreducible: true */
/* convention: sysv-amd64 (score 0.70) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 9 */
/* struct_layouts: pointer=6 stack=1 */
/* switch_tables: 0 */
int64_t _pei386_runtime_relocator(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    int64_t v6 = 0LL;
    void * v7 = ((void *)(0LL));
    int64_t v8 = 0LL;
    void * v9 = ((void *)(0LL));
    int64_t v10 = 0LL;
    void * v11 = ((void *)(0LL));
    int64_t v12 = arg0;
    void * v13 = ((void *)(0LL));
    int64_t v14 = arg1;
    void * v15 = ((void *)(0LL));
    int64_t v16 = 0LL;
    void * v17 = ((void *)(0LL));
    void * v18 = ((void *)(0LL));
    void * v19 = ((void *)(0LL));
    int32_t v20 = 0LL;
    int64_t src = 0LL;
    int64_t v22 = 0LL;
    int8_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    void * v26 = ((void *)(0LL));
    S_140001a00_v27_t * v27 = ((S_140001a00_v27_t *)(0LL));
    int64_t v28 = 0LL;
    void * v29 = ((void *)(0LL));
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    void * v32 = ((void *)(0LL));
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    void * v35 = ((void *)(0LL));
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    void * v38 = ((void *)(0LL));
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    void * v41 = ((void *)(0LL));
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    void * v44 = ((void *)(0LL));
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    void * v47 = ((void *)(0LL));
    int64_t v48 = 0LL;
    int64_t v49 = 0LL;
    void * v50 = ((void *)(0LL));
    int64_t v51 = 0LL;
    int64_t v52 = arg2;
    int64_t v53 = arg3;
    int64_t v54 = arg4;
    int64_t v55 = arg5;
    int64_t v56 = 0LL;
    int64_t v57 = 0LL;
    int64_t v58 = 0LL;
    int64_t v59 = 0LL;
    int64_t v60 = 0LL;
    int64_t v61 = 0LL;
    int64_t v62 = 0LL;
    int64_t v63 = 0LL;
    int64_t v64 = 0LL;
    int64_t v65 = 0LL;
    int64_t dst = 0LL;
    int64_t v67 = 0LL;
    S_140001a00_v68_t * v68 = ((S_140001a00_v68_t *)(0LL));
    void * v69 = ((void *)(0LL));
    void * v70 = ((void *)(0LL));
    int64_t v71 = 0LL;
    int64_t v72 = 0LL;
    int64_t v73 = 0LL;
    int8_t v74 = 0LL;
    int8_t v75 = 0LL;
    S_140001a00_v76_t * v76 = ((S_140001a00_v76_t *)(0LL));
    int64_t v77 = 0LL;
    int64_t v78 = 0LL;
    int32_t v79 = 0LL;
    int32_t v80 = 0LL;
    int32_t v81 = 0LL;
    int8_t v82 = 0LL;
    int64_t v83 = 0LL;
    int32_t v84 = 0LL;
    int32_t v85 = 0LL;
    int32_t v86 = 0LL;
    int8_t v87 = 0LL;
    int64_t v88 = 0LL;
    int32_t v89 = 0LL;
    int64_t v90 = 0LL;
    int64_t v91 = 0LL;
    int64_t v92 = 0LL;
    int32_t v93 = 0LL;
    int32_t v94 = 0LL;
    int8_t v95 = 0LL;
    int64_t v96 = 0LL;
    int64_t v97 = 0LL;
    int64_t v98 = 0LL;
    void * v99 = ((void *)(0LL));
    void * v100 = ((void *)(0LL));
    int8_t v101 = 0LL;
    S_140001a00_v102_t * v102 = ((S_140001a00_v102_t *)(0LL));
    int32_t v103 = 0LL;
    int32_t v104 = 0LL;
    int64_t v105 = 0LL;
    int32_t v106 = 0LL;
    int32_t v107 = 0LL;
    int64_t v108 = 0LL;
    int32_t v109 = 0LL;
    int32_t v110 = 0LL;
    void * v111 = ((void *)(0LL));
    int32_t v112 = 0LL;
    int64_t v113 = 0LL;
    int64_t v114 = 0LL;
    void * v115 = ((void *)(0LL));
    int8_t v116 = 0LL;
    void * v117 = ((void *)(0LL));
    int64_t v118 = 0LL;
    int8_t v119 = 0LL;
    int8_t v120 = 0LL;
    int8_t v121 = 0LL;
    int16_t v122 = 0LL;
    int16_t v123 = 0LL;
    int16_t v124 = 0LL;
    int16_t v125 = 0LL;
    void * v126 = ((void *)(0LL));
    int64_t v127 = 0LL;
    int32_t v128 = 0LL;
    void * v129 = ((void *)(0LL));
    int8_t v130 = 0LL;
    int8_t v131 = 0LL;
    void * v132 = ((void *)(0LL));
    void * v133 = ((void *)(0LL));
    int64_t v134 = 0LL;
    int64_t v135 = 0LL;
    int64_t n = 0LL;
    void * v137 = ((void *)(0LL));
    void * v138 = ((void *)(0LL));
    int8_t v139 = 0LL;
    int8_t v140 = 0LL;
    int8_t v141 = 0LL;
    int8_t v142 = 0LL;
    void * v143 = ((void *)(0LL));
    int64_t v144 = 0LL;
    int32_t v145 = 0LL;
    void * v146 = ((void *)(0LL));
    int8_t v147 = 0LL;
    int8_t v148 = 0LL;
    void * v149 = ((void *)(0LL));
    void * v150 = ((void *)(0LL));
    int64_t v151 = 0LL;
    int64_t v152 = 0LL;
    int64_t n_1 = 0LL;
    void * v154 = ((void *)(0LL));
    void * v155 = ((void *)(0LL));
    int8_t v156 = 0LL;
    int64_t v157 = 0LL;
    int64_t v158 = 0LL;
    int64_t v159 = 0LL;
    int64_t v160 = 0LL;
    int32_t v161 = 0LL;
    void * v162 = ((void *)(0LL));
    void * v163 = ((void *)(0LL));
    int64_t v164 = 0LL;
    void * v165 = ((void *)(0LL));
    int64_t v166 = 0LL;
    int64_t v167 = 0LL;
    int64_t n_2 = 0LL;
    void * v169 = ((void *)(0LL));
    void * v170 = ((void *)(0LL));
    int8_t v171 = 0LL;
    int64_t v172 = 0LL;
    void * v173 = ((void *)(0LL));
    int64_t v174 = 0LL;
    int32_t v175 = 0LL;
    int32_t v176 = 0LL;
    int64_t v177 = 0LL;
    int32_t v178 = 0LL;
    int32_t v179 = 0LL;
    void * v180 = ((void *)(0LL));
    int64_t v181 = 0LL;
    int32_t v182 = 0LL;
    void * v183 = ((void *)(0LL));
    int64_t v184 = 0LL;
    int8_t v185 = 0LL;
    int8_t v186 = 0LL;
    void * v187 = ((void *)(0LL));
    void * v188 = ((void *)(0LL));
    int64_t v189 = 0LL;
    int64_t v190 = 0LL;
    int64_t n_3 = 0LL;
    void * v192 = ((void *)(0LL));
    void * v193 = ((void *)(0LL));
    int64_t v194 = 0LL;
    void * v195 = ((void *)(0LL));
    void * v196 = ((void *)(0LL));
    int64_t v197 = 0LL;
    int64_t v198 = 0LL;
    int32_t v199 = 0LL;
    int64_t v200 = 0LL;
    int64_t v201 = 0LL;
    int64_t v202 = 0LL;
    int64_t v203 = 0LL;
    void * v204 = ((void *)(0LL));
    int64_t v205 = 0LL;
    int32_t v206 = 0LL;
    int32_t v207 = 0LL;
    int32_t v208 = 0LL;
    int8_t v209 = 0LL;
    int64_t v210 = 0LL;
    int64_t v211 = 0LL;
    int64_t v212 = 0LL;
    int64_t i = 0LL;
    int64_t v214 = 0LL;
    int64_t v215 = 0LL;
    int64_t v216 = 0LL;
    S_140001a00_v217_t * v217 = ((S_140001a00_v217_t *)(0LL));
    int32_t v218 = 0LL;
    int32_t v219 = 0LL;
    int32_t v220 = 0LL;
    int8_t v221 = 0LL;
    int64_t v222 = 0LL;
    int64_t v223 = 0LL;
    int64_t v224 = 0LL;
    int64_t v225 = 0LL;
    int64_t v226 = 0LL;
    int64_t v227 = 0LL;
    void * v228 = ((void *)(0LL));
    int64_t v229 = 0LL;
    int64_t v230 = 0LL;
    int64_t v231 = 0LL;
    int64_t v232 = 0LL;
    int32_t v233 = 0LL;
    int8_t v234 = 0LL;
    int32_t v235 = 0LL;
    int32_t v236 = 0LL;
    int32_t v237 = 0LL;
    int8_t v238 = 0LL;
    int64_t v239 = 0LL;
    int32_t v240 = 0LL;
    int32_t v241 = 0LL;
    int32_t v242 = 0LL;
    int8_t v243 = 0LL;
    int64_t v244 = 0LL;
    int32_t v245 = 0LL;
    int32_t v246 = 0LL;
    int32_t v247 = 0LL;
    int8_t v248 = 0LL;
    int64_t v249 = 0LL;
    int64_t v250 = 0LL;
    int32_t v251 = 0LL;
    int64_t v252 = 0LL;
    int64_t v253 = 0LL;
    int64_t v254 = 0LL;
    int8_t v255 = 0LL;
    int64_t v256 = 0LL;
    int64_t v257 = 0LL;
    void * v258 = ((void *)(0LL));
    void * v259 = ((void *)(0LL));
    S_140001a00_v260_t * v260 = ((S_140001a00_v260_t *)(0LL));
    int64_t v261 = 0LL;
    int64_t v262 = 0LL;
    int64_t v263 = 0LL;
    int32_t v264 = 0LL;
    int32_t v265 = 0LL;
    int32_t v266 = 0LL;
    int32_t v267 = 0LL;
    int64_t v268 = 0LL;
    void * v269 = ((void *)(0LL));
    int32_t v270 = 0LL;
    int32_t v271 = 0LL;
    int64_t v272 = 0LL;
    int64_t v273 = 0LL;
    void * v274 = ((void *)(0LL));
    int64_t v275 = 0LL;
    int64_t v276 = 0LL;
    int64_t n_4 = 0LL;
    int64_t v278 = 0LL;
    int64_t v279 = 0LL;
    void * v280 = ((void *)(0LL));
    int8_t v281 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 8LL)));
    *((int64_t *)(((int64_t)(v5)))) = v6;
    v7 = ((void *)((((int64_t)(v5)) - 8LL)));
    *((int64_t *)(((int64_t)(v7)))) = v8;
    v9 = ((void *)((((int64_t)(v7)) - 8LL)));
    *((int64_t *)(((int64_t)(v9)))) = v10;
    v11 = ((void *)((((int64_t)(v9)) - 8LL)));
    *((int64_t *)(((int64_t)(v11)))) = v12;
    v13 = ((void *)((((int64_t)(v11)) - 8LL)));
    *((int64_t *)(((int64_t)(v13)))) = v14;
    v15 = ((void *)((((int64_t)(v13)) - 8LL)));
    *((int64_t *)(((int64_t)(v15)))) = v16;
    v17 = ((void *)((((int64_t)(v15)) - 72LL)));
    v18 = ((void *)((((int64_t)(v17)) + 64LL)));
    v19 = ((void *)(((int64_t)(v18))));
    v20 = (*((int32_t *)(5368742144LL)));
    src = v20;
    v22 = (src & src);
    v23 = (v22 == 0LL);
    if (v23) {
        *((int32_t *)(5368742144LL)) = 1LL;
        v56 = ((long long (*)(long long, long long, long long, long long, long long, long long))__mingw_GetSectionCount)(v12, src, v52, v53, v54, v55);
        (/* opaque: cdqe */ 0);
        v57 = (v56 * 4LL);
        v58 = (v56 + v57);
        v59 = v58;
        v60 = (v59 * 8LL);
        v61 = (15LL + v60);
        v62 = v61;
        v63 = (v62 & -16LL);
        v64 = ((long long (*)(long long, long long, long long, long long, long long, long long))fn_1400027e0)(v12, src, v52, v53, v54, v55);
        v65 = (*((int64_t *)(5368726544LL)));
        dst = v65;
        v67 = (*((int64_t *)(5368726560LL)));
        v68 = ((S_140001a00_v68_t *)(v67));
        v69 = ((void *)((((int64_t)(v17)) - v64)));
        *((int32_t *)(5368742148LL)) = 0LL;
        v70 = ((void *)((((int64_t)(v69)) + 48LL)));
        v71 = ((int64_t)(v70));
        *((int64_t *)(5368742152LL)) = v71;
        v72 = dst;
        v73 = (v72 - ((int64_t)(v68)));
        v74 = (v73 <= 7LL);
        if (v74) {
L0:;
            /* phi v25 <- (bb0: v24) (bb4: v73) (bb10: v89) (bb31: v207) (bb36: v230) (bb40: v251) */
            v26 = ((void *)((((int64_t)(v19)) + 8LL)));
            v27 = ((S_140001a00_v27_t *)(((int64_t)(v26))));
            v28 = (*((int64_t *)(((int64_t)(v27)))));
            v29 = ((void *)((((int64_t)(v27)) + 8LL)));
            v30 = v28;
            v31 = v27->field_8;
            v32 = ((void *)((((int64_t)(v29)) + 8LL)));
            v33 = v31;
            v34 = (*((int64_t *)(((int64_t)(v32)))));
            v35 = ((void *)((((int64_t)(v32)) + 8LL)));
            v36 = v34;
            v37 = (*((int64_t *)(((int64_t)(v35)))));
            v38 = ((void *)((((int64_t)(v35)) + 8LL)));
            v39 = v37;
            v40 = (*((int64_t *)(((int64_t)(v38)))));
            v41 = ((void *)((((int64_t)(v38)) + 8LL)));
            v42 = v40;
            v43 = (*((int64_t *)(((int64_t)(v41)))));
            v44 = ((void *)((((int64_t)(v41)) + 8LL)));
            v45 = v43;
            v46 = (*((int64_t *)(((int64_t)(v44)))));
            v47 = ((void *)((((int64_t)(v44)) + 8LL)));
            v48 = v46;
            v49 = (*((int64_t *)(((int64_t)(v47)))));
            v50 = ((void *)((((int64_t)(v47)) + 8LL)));
            v51 = v49;
            return v25;
        } else {
            v75 = (v73 > 11LL);
            if (v75) {
                v235 = (*((int32_t *)(((int64_t)(v68)))));
                v236 = v235;
                v237 = (v236 & v236);
                v238 = (v237 != 0LL);
                if (v238) {
L2:;
                    /* phi v250 <- (bb6: v76) (bb7: v76) (bb38: v68) (bb39: v68) */
                    /* phi v251 <- (bb6: v73) (bb7: v85) (bb38: v73) (bb39: v73) */
                    /* phi v252 <- (bb6: v80) (bb7: v80) (bb38: v52) (bb39: v52) */
                    /* phi v253 <- (bb6: v77) (bb7: v77) (bb38: v54) (bb39: v241) */
                    /* phi v254 <- (bb6: v78) (bb7: v78) (bb38: v236) (bb39: v236) */
                    v255 = (v250 >= dst);
                    if (v255) {
                    } else {
                        v256 = (*((int64_t *)(5368726528LL)));
                        v257 = v256;
                        v258 = ((void *)((((int64_t)(v19)) + -8LL)));
                        v259 = ((void *)(((int64_t)(v258))));
                        while (1) {
                            /* phi v260 <- (bb41: v250) (bb44: v268) */
                            /* phi v261 <- (bb41: v252) (bb44: n_4) */
                            /* phi v262 <- (bb41: v253) (bb44: v276) */
                            v263 = (((int64_t)(v260)) + 4LL);
                            v264 = v260->field_4;
                            v265 = v264;
                            v266 = (*((int32_t *)(((int64_t)(v260)))));
                            v267 = v266;
                            v268 = (((int64_t)(v260)) + 8LL);
                            v269 = ((void *)((v257 + v265)));
                            v270 = (*((int32_t *)(((int64_t)(v269)))));
                            v271 = (v267 + v270);
                            v272 = (v265 + v257);
                            v273 = v272;
                            v274 = ((void *)((((int64_t)(v19)) + -8LL)));
                            *((int32_t *)(((int64_t)(v274)))) = v271;
                            v275 = ((long long (*)(long long, long long, long long, long long, long long, long long))mark_section_writable)(dst, src, v261, v273, v262, v254);
                            v276 = 4LL;
                            n_4 = ((int64_t)(v259));
                            v278 = (v265 + v257);
                            v279 = v278;
                            v280 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))memcpy)(dst, src, n_4, v279, v276, v254)));
                            v281 = (v268 < dst);
                            if (v281) {
                                continue;
                            } else {
                                break;
                            }
                        }
L1:;
                        /* phi v204 <- (bb21: v117) (bb30: v165) (bb45: v259) */
                        /* phi v205 <- (bb21: v118) (bb30: v164) (bb45: v268) */
                        v206 = (*((int32_t *)(5368742148LL)));
                        v207 = v206;
                        v208 = (v207 & v207);
                        v209 = (v208 <= 0LL);
                        if (v209) {
                        } else {
                            v210 = (*((int64_t *)(5368746736LL)));
                            v211 = v210;
                            v212 = (v205 ^ v205);
                            while (1) {
                                /* phi i <- (bb32: src) (bb35: v231) */
                                /* phi v214 <- (bb32: v212) (bb35: v232) */
                                v215 = (*((int64_t *)(5368742152LL)));
                                v216 = v215;
                                v217 = ((S_140001a00_v217_t *)((v216 + v214)));
                                v218 = (*((int32_t *)(((int64_t)(v217)))));
                                v219 = v218;
                                v220 = (v219 & v219);
                                v221 = (v220 == 0LL);
                                if (v221) {
                                } else {
                                    v222 = (((int64_t)(v217)) + 16LL);
                                    v223 = v217->field_10;
                                    v224 = v223;
                                    v225 = (((int64_t)(v217)) + 8LL);
                                    v226 = v217->field_8;
                                    v227 = v226;
                                    v228 = ((void *)(((int64_t)(v204))));
                                    v229 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v211, i, v224, v227, v219, ((int64_t)(v228)));
                                }
                                /* phi v230 <- (bb33: v217) (bb34: v229) */
                                v231 = (i + 1LL);
                                v232 = (v214 + 40LL);
                                v233 = (*((int32_t *)(5368742148LL)));
                                v234 = (v231 < v233);
                                if (v234) {
                                    continue;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    goto L0;
                } else {
                    v239 = (((int64_t)(v68)) + 4LL);
                    v240 = v68->field_4;
                    v241 = v240;
                    v242 = (v241 & v241);
                    v243 = (v242 == 0LL);
                    if (v243) {
                        v244 = (((int64_t)(v68)) + 8LL);
                        v245 = v68->field_8;
                        v246 = v245;
                        v247 = (v246 & v246);
                        v248 = (v247 != 0LL);
                        if (v248) {
L3:;
                            /* phi v88 <- (bb7: v76) (bb64: v68) */
                            /* phi v89 <- (bb7: v85) (bb64: v73) */
                            /* phi v90 <- (bb7: v77) (bb64: v241) */
                            /* phi v91 <- (bb7: v78) (bb64: v236) */
                            v92 = (v88 + 8LL);
                            v93 = (*((int32_t *)(v92)));
                            v94 = v93;
                            v95 = (v94 != 1LL);
                            if (v95) {
                                /* phi v199 <- (bb8: v94) (bb67: v112) */
                                /* phi v200 <- (bb8: v90) (bb67: v196) */
                                /* phi v201 <- (bb8: v91) (bb67: v114) */
                                v202 = 5368726176LL;
                                v203 = ((long long (*)(long long, long long, long long, long long, long long, long long))__report_error)(dst, src, v199, v202, v200, v201);
                                /* structurally unreachable: block 69 */
                                __builtin_unreachable();
                            } else {
                                v96 = (v88 + 12LL);
                                v97 = (*((int64_t *)(5368726528LL)));
                                v98 = v97;
                                v99 = ((void *)((((int64_t)(v19)) + -8LL)));
                                v100 = ((void *)(((int64_t)(v99))));
                                v101 = (v96 < dst);
                                if (v101) {
                                    while (1) {
                                        /* phi v102 <- (bb9: v96) (bb21: v118) (bb29: v164) */
                                        v103 = (*((int32_t *)(((int64_t)(v102)))));
                                        v104 = v103;
                                        v105 = (((int64_t)(v102)) + 8LL);
                                        v106 = v102->field_8;
                                        v107 = v106;
                                        v108 = (((int64_t)(v102)) + 4LL);
                                        v109 = v102->field_4;
                                        v110 = v109;
                                        v111 = ((void *)((v104 + v98)));
                                        v112 = v107;
                                        v113 = (*((int64_t *)(((int64_t)(v111)))));
                                        v114 = v113;
                                        v115 = ((void *)((v110 + v98)));
                                        v116 = (v112 == 32LL);
                                        if (v116) {
                                            v175 = (*((int32_t *)(((int64_t)(v115)))));
                                            v176 = v175;
                                            /* structurally unreachable: block 47 */
                                            __builtin_unreachable();
                                        } else {
                                            /* structurally unreachable: block 23 */
                                            __builtin_unreachable();
                                        }
                                    }
                                    goto L1;
                                } else {
                                    goto L0;
                                }
                            }
                        } else {
                            v249 = (((int64_t)(v68)) + 12LL);
L4:;
                            /* phi v76 <- (bb5: v68) (bb65: v249) */
                            /* phi v77 <- (bb5: v54) (bb65: v241) */
                            /* phi v78 <- (bb5: v55) (bb65: v236) */
                            v79 = (*((int32_t *)(((int64_t)(v76)))));
                            v80 = v79;
                            v81 = (v80 & v80);
                            v82 = (v81 != 0LL);
                            if (v82) {
                                goto L2;
                            } else {
                                v83 = (((int64_t)(v76)) + 4LL);
                                v84 = v76->field_4;
                                v85 = v84;
                                v86 = (v85 & v85);
                                v87 = (v86 != 0LL);
                                if (v87) {
                                    goto L2;
                                } else {
                                    goto L3;
                                }
                            }
                        }
                    } else {
                        goto L2;
                    }
                }
            } else {
                goto L4;
            }
        }
    } else {
        goto L0;
    }
}

/* dac-recovered function */
/* address: 0x140001d90 */
/* end: 0x140001dd0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 4 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.85) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 2 */
/* struct_layouts: pointer=1 stack=1 */
/* switch_tables: 0 */
int64_t __mingw_raise_matherr(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    S_140001d90_v1_t * v1 = ((S_140001d90_v1_t *)(0LL));
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int8_t v5 = 0LL;
    void * v6 = ((void *)(0LL));
    int64_t v7 = arg3;
    int64_t v8 = 0LL;
    void * v9 = ((void *)(0LL));
    void * v10 = ((void *)(0LL));
    int64_t v11 = arg2;
    int64_t v12 = arg0;
    int64_t v13 = arg1;
    int64_t v14 = arg4;
    int64_t v15 = arg5;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;

    v1 = ((S_140001d90_v1_t *)((v0 - 88LL)));
    v2 = (*((int64_t *)(5368742160LL)));
    v3 = v2;
    v4 = (v3 & v3);
    v5 = (v4 == 0LL);
    if (v5) {
    } else {
        (/* opaque: movsd */ 0);
        (/* opaque: unpcklpd */ 0);
        v6 = ((void *)((((int64_t)(v1)) + 32LL)));
        v1->field_20 = v7;
        v9 = ((void *)(((int64_t)(v6))));
        v10 = ((void *)((((int64_t)(v1)) + 40LL)));
        v1->field_28 = v11;
        (/* opaque: movaps */ 0);
        (/* opaque: movsd */ 0);
        v16 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v12, v13, v11, ((int64_t)(v9)), v14, v15);
    }
    /* phi v17 <- (bb0: v3) (bb2: v16) */
    v18 = (((int64_t)(v1)) + 88LL);
    return v17;
}

/* dac-recovered function */
/* address: 0x140001dd0 */
/* end: 0x140001de0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.48) */
/* args: rcx */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void __mingw_setusermatherr(int64_t arg0) {
    int64_t v0 = arg0;

    *((int64_t *)(5368742160LL)) = v0;
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140001de0 */
/* end: 0x140001f80 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 18 */
/* goto_count: 7 */
/* label_count: 3 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.85) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=1 stack=0 */
/* switch_tables: 1 */
int64_t __mingw_SEH_error_handler(int64_t arg0, int64_t arg1, int64_t arg2, S_140001de0_v2_t * arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    S_140001de0_v2_t * v2 = arg3;
    int64_t v3 = 0LL;
    int8_t v4 = 0LL;
    int8_t v5 = 0LL;
    int8_t v6 = 0LL;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int8_t v10 = 0LL;
    int32_t v11 = 0LL;
    int32_t v12 = 0LL;
    int8_t v13 = 0LL;
    int8_t v14 = 0LL;
    int32_t v15 = 0LL;
    int8_t v16 = 0LL;
    int64_t v17 = 0LL;
    int32_t v18 = 0LL;
    int64_t v19 = 0LL;
    int32_t v20 = 0LL;
    int32_t v21 = 0LL;
    int32_t v22 = 0LL;
    int8_t v23 = 0LL;
    int8_t v24 = 0LL;
    int8_t v25 = 0LL;
    int64_t v26 = arg2;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = arg0;
    int64_t v30 = arg1;
    int64_t v31 = arg4;
    int64_t v32 = arg5;
    int64_t v33 = 0LL;
    int8_t v34 = 0LL;
    int64_t v35 = 0LL;
    int8_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int32_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int8_t v46 = 0LL;
    int64_t v47 = 0LL;
    int8_t v48 = 0LL;
    int64_t v49 = 0LL;
    int64_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int64_t v53 = 0LL;
    int64_t v54 = 0LL;
    int64_t v55 = 0LL;
    int64_t v56 = 0LL;
    int64_t v57 = 0LL;
    int64_t v58 = 0LL;

    v1 = (v0 - 40LL);
    v3 = (((int64_t)(v2)) + 4LL);
    v4 = v2->field_4;
    v5 = (v4 & 2LL);
    v6 = (v5 != 0LL);
    if (v6) {
L1:;
        v58 = 1LL;
L0:;
        /* phi v56 <- (bb16: v55) (bb34: v58) (bb36: v42) */
        v57 = (v1 + 40LL);
        return v56;
    } else {
        v7 = 4848615423LL;
        v8 = (*((int64_t *)(((int64_t)(v2)))));
        v9 = (v7 & v8);
        v10 = (v9 == 541541187LL);
        if (v10) {
L2:;
            /* phi v54 <- (bb1: v9) (bb4: v15) (bb9: v12) (bb15: v38) (bb28: v50) (bb38: v53) (bb41: v41) */
            v55 = (v54 ^ v54);
            goto L0;
        } else {
            v11 = (*((int32_t *)(((int64_t)(v2)))));
            v12 = v11;
            v13 = (v12 > -1073741674LL);
            if (v13) {
                goto L1;
            } else {
                v14 = (v12 <= -1073741685LL);
                if (v14) {
                    v23 = (v12 == -1073741819LL);
                    if (v23) {
                        v43 = (v26 ^ v26);
                        v44 = 11LL;
                        v45 = ((long long (*)(long long, long long, long long, long long, long long, long long))signal)(v29, v30, v43, v44, v31, v32);
                        v46 = (v45 == 1LL);
                        if (v46) {
                            v51 = 1LL;
                            v52 = 11LL;
                            v53 = ((long long (*)(long long, long long, long long, long long, long long, long long))signal)(v29, v30, v51, v52, v31, v32);
                            goto L2;
                        } else {
                            v47 = (v45 & v45);
                            v48 = (v47 == 0LL);
                            if (v48) {
                                goto L1;
                            } else {
                                v49 = 11LL;
                                v50 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v29, v30, v43, v49, v31, v32);
                                goto L2;
                            }
                        }
                    } else {
                        /* structurally unreachable: block 8 */
                        __builtin_unreachable();
                    }
                    goto L0;
                } else {
                    v15 = (v12 + 1073741683LL);
                    v16 = (v15 > 9LL);
                    if (v16) {
                        goto L2;
                    } else {
                        v17 = 5368726368LL;
                        v18 = (v15 * 4LL);
                        v19 = (v17 + v18);
                        v20 = (*((int32_t *)(v19)));
                        v21 = v20;
                        v22 = (v21 + v17);
                        /* recovered switch table at block 5 */
                        switch (v15) {
                            case 5LL: {
                                goto L3;
                            }
                            case 9LL: {
                                goto L4;
                            }
                            default: {
                                /* structurally unreachable: block 5 */
                                __builtin_unreachable();
                            }
                        }
                    }
                }
            }
        }
    }
L3:;
L4:;
}

/* dac-recovered function */
/* address: 0x140001f80 */
/* end: 0x140002140 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 20 */
/* goto_count: 5 */
/* label_count: 3 */
/* irreducible: false */
/* convention: ms-x64 (score 0.43) */
/* args: rcx */
/* return_reg: rax */
/* stack_locals: 1 */
/* struct_layouts: pointer=1 stack=0 */
/* switch_tables: 1 */
int64_t _gnu_exception_handler(int64_t arg0) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = arg0;
    int64_t v5 = 0LL;
    S_140001f80_v6_t * v6 = ((S_140001f80_v6_t *)(0LL));
    int32_t v7 = 0LL;
    int32_t v8 = 0LL;
    int64_t v9 = 0LL;
    int32_t v10 = 0LL;
    int32_t v11 = 0LL;
    int8_t v12 = 0LL;
    int8_t v13 = 0LL;
    int8_t v14 = 0LL;
    int32_t v15 = 0LL;
    int8_t v16 = 0LL;
    int64_t v17 = 0LL;
    int32_t v18 = 0LL;
    int64_t v19 = 0LL;
    int32_t v20 = 0LL;
    int32_t v21 = 0LL;
    int32_t v22 = 0LL;
    int8_t v23 = 0LL;
    int8_t v24 = 0LL;
    int8_t v25 = 0LL;
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int8_t v33 = 0LL;
    int64_t v34 = 0LL;
    int8_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int8_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int8_t v45 = 0LL;
    int64_t v46 = 0LL;
    int8_t v47 = 0LL;
    int64_t v48 = 0LL;
    int64_t v49 = 0LL;
    int64_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int64_t v53 = 0LL;
    int64_t v54 = 0LL;
    int64_t v55 = 0LL;
    int8_t v56 = 0LL;
    int64_t v57 = 0LL;
    void * v58 = ((void *)(0LL));
    int64_t v59 = 0LL;
    void * v60 = ((void *)(0LL));
    int64_t v61 = 0LL;
    int64_t v62 = 0LL;
    void * v63 = ((void *)(0LL));
    int64_t v64 = 0LL;
    void * v65 = ((void *)(0LL));
    int64_t v66 = 0LL;
    int64_t v67 = 0LL;
    void * v68 = ((void *)(0LL));
    int64_t v69 = 0LL;
    void * v70 = ((void *)(0LL));
    int64_t v71 = 0LL;
    int64_t v72 = 0LL;
    int8_t v73 = 0LL;
    int8_t v74 = 0LL;
    int8_t v75 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 32LL)));
    v5 = (*((int64_t *)(v4)));
    v6 = ((S_140001f80_v6_t *)(v5));
    v7 = (*((int32_t *)(((int64_t)(v6)))));
    v8 = v7;
    v9 = v4;
    v10 = v8;
    v11 = (v10 & 553648127LL);
    v12 = (v11 == 541541187LL);
    if (v12) {
        v72 = (((int64_t)(v6)) + 4LL);
        v73 = v6->field_4;
        v74 = (v73 & 1LL);
        v75 = (v74 != 0LL);
        if (v75) {
L2:;
            v13 = (v8 > -1073741674LL);
            if (v13) {
L0:;
                v53 = (*((int64_t *)(5368742192LL)));
                v54 = v53;
                v55 = (v54 & v54);
                v56 = (v55 == 0LL);
                if (v56) {
                    v62 = (v54 ^ v54);
                    v63 = ((void *)((((int64_t)(v3)) + 32LL)));
                    v64 = (*((int64_t *)(((int64_t)(v63)))));
                    v65 = ((void *)((((int64_t)(v63)) + 8LL)));
                    v66 = v64;
                    return v62;
                } else {
                    v57 = v9;
                    v58 = ((void *)((((int64_t)(v3)) + 32LL)));
                    v59 = (*((int64_t *)(((int64_t)(v58)))));
                    v60 = ((void *)((((int64_t)(v58)) + 8LL)));
                    v61 = v59;
                    /* structurally unreachable: block 9 */
                    __builtin_unreachable();
                }
            } else {
                v14 = (v8 <= -1073741685LL);
                if (v14) {
                    v23 = (v8 == -1073741819LL);
                    if (v23) {
                        v42 = (((int64_t)(v6)) ^ ((int64_t)(v6)));
                        v43 = 11LL;
                        v44 = ((long long (*)(long long, long long, long long, long long, long long, long long))signal)(v28, v29, v42, v43, v30, v31);
                        v45 = (v44 == 1LL);
                        if (v45) {
                            v50 = 1LL;
                            v51 = 11LL;
                            v52 = ((long long (*)(long long, long long, long long, long long, long long, long long))signal)(v28, v29, v50, v51, v30, v31);
L1:;
                            v67 = -1LL;
                            v68 = ((void *)((((int64_t)(v3)) + 32LL)));
                            v69 = (*((int64_t *)(((int64_t)(v68)))));
                            v70 = ((void *)((((int64_t)(v68)) + 8LL)));
                            v71 = v69;
                            return v67;
                        } else {
                            v46 = (v44 & v44);
                            v47 = (v46 == 0LL);
                            if (v47) {
                                goto L0;
                            } else {
                                v48 = 11LL;
                                v49 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v28, v29, v42, v48, v30, v31);
                                goto L1;
                            }
                        }
                    } else {
                        /* structurally unreachable: block 12 */
                        __builtin_unreachable();
                    }
                } else {
                    v15 = (v8 + 1073741683LL);
                    v16 = (v15 > 9LL);
                    if (v16) {
                        goto L1;
                    } else {
                        v17 = 5368726408LL;
                        v18 = (v15 * 4LL);
                        v19 = (v17 + v18);
                        v20 = (*((int32_t *)(v19)));
                        v21 = v20;
                        v22 = (v21 + v17);
                        /* recovered switch table at block 4 */
                        switch (v15) {
                            case 5LL: {
                                goto L3;
                            }
                            case 9LL: {
                                goto L4;
                            }
                            default: {
                                /* structurally unreachable: block 4 */
                                __builtin_unreachable();
                            }
                        }
                    }
                }
            }
        } else {
            goto L1;
        }
    } else {
        goto L2;
    }
L3:;
L4:;
}

/* dac-recovered function */
/* address: 0x140002140 */
/* end: 0x1400021b0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 10 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.35) */
/* args: rdi,rsi,rdx */
/* return_reg: none */
/* stack_locals: 4 */
/* struct_layouts: pointer=1 stack=1 */
/* switch_tables: 0 */
void __mingwthr_run_key_dtors_part_0(int64_t arg0, int64_t arg1, int64_t arg2) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = arg0;
    void * v5 = ((void *)(0LL));
    int64_t v6 = arg1;
    void * v7 = ((void *)(0LL));
    int64_t v8 = 0LL;
    void * v9 = ((void *)(0LL));
    int64_t v10 = 0LL;
    int64_t v11 = arg2;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int8_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    S_140002140_v24_t * v24 = ((S_140002140_v24_t *)(0LL));
    int32_t v25 = 0LL;
    int32_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int8_t v31 = 0LL;
    int64_t v32 = 0LL;
    int8_t v33 = 0LL;
    int64_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int64_t v42 = 0LL;
    int8_t v43 = 0LL;
    int64_t v44 = 0LL;
    void * v45 = ((void *)(0LL));
    int64_t v46 = 0LL;
    void * v47 = ((void *)(0LL));
    int64_t v48 = 0LL;
    int64_t v49 = 0LL;
    void * v50 = ((void *)(0LL));
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    void * v53 = ((void *)(0LL));
    int64_t v54 = 0LL;
    int64_t v55 = 0LL;
    void * v56 = ((void *)(0LL));
    int64_t v57 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 8LL)));
    *((int64_t *)(((int64_t)(v5)))) = v6;
    v7 = ((void *)((((int64_t)(v5)) - 8LL)));
    *((int64_t *)(((int64_t)(v7)))) = v8;
    v9 = ((void *)((((int64_t)(v7)) - 40LL)));
    v10 = 5368742240LL;
    v14 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v4, v6, v11, v10, v12, v13);
    v15 = (*((int64_t *)(5368742208LL)));
    v16 = v15;
    v17 = (v16 & v16);
    v18 = (v17 == 0LL);
    if (v18) {
    } else {
        v19 = (*((int64_t *)(5368746728LL)));
        v20 = v19;
        v21 = (*((int64_t *)(5368746656LL)));
        v22 = v21;
        while (1) {
            /* phi v23 <- (bb2: v6) (bb8: v28) */
            /* phi v24 <- (bb2: v16) (bb8: v41) */
            v25 = (*((int32_t *)(((int64_t)(v24)))));
            v26 = v25;
            v27 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v22, v23, v11, v26, v12, v13);
            v28 = v27;
            v29 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v22, v28, v11, v26, v12, v13);
            v30 = (v28 & v28);
            v31 = (v30 == 0LL);
            if (v31) {
            } else {
                v32 = (v29 & v29);
                v33 = (v32 != 0LL);
                if (v33) {
                } else {
                    v34 = (((int64_t)(v24)) + 8LL);
                    v35 = v24->field_8;
                    v36 = v35;
                    v37 = v28;
                    v38 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v22, v28, v11, v37, v12, v13);
                }
            }
            v39 = (((int64_t)(v24)) + 16LL);
            v40 = v24->field_10;
            v41 = v40;
            v42 = (v41 & v41);
            v43 = (v42 != 0LL);
            if (v43) {
                continue;
            }
        }
    }
    v44 = 5368742240LL;
    v45 = ((void *)((((int64_t)(v9)) + 40LL)));
    v46 = (*((int64_t *)(((int64_t)(v45)))));
    v47 = ((void *)((((int64_t)(v45)) + 8LL)));
    v48 = v46;
    v49 = (*((int64_t *)(((int64_t)(v47)))));
    v50 = ((void *)((((int64_t)(v47)) + 8LL)));
    v51 = v49;
    v52 = (*((int64_t *)(((int64_t)(v50)))));
    v53 = ((void *)((((int64_t)(v50)) + 8LL)));
    v54 = v52;
    v55 = (*((int64_t *)(((int64_t)(v53)))));
    v56 = ((void *)((((int64_t)(v53)) + 8LL)));
    v57 = v55;
    /* structurally unreachable: block 9 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1400021b0 */
/* end: 0x140002240 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 9 */
/* goto_count: 1 */
/* label_count: 1 */
/* irreducible: false */
/* convention: ms-x64 (score 0.88) */
/* args: rcx,rdx,r8 */
/* return_reg: rax */
/* stack_locals: 3 */
/* struct_layouts: pointer=2 stack=0 */
/* switch_tables: 0 */
int64_t ___w64_mingwthr_add_key_dtor(int64_t arg0, int64_t arg1, int64_t arg2) {
    int64_t v0 = 0LL;
    S_1400021b0_v1_t * v1 = ((S_1400021b0_v1_t *)(0LL));
    int32_t v2 = 0LL;
    int32_t v3 = 0LL;
    int64_t v4 = arg0;
    int64_t v5 = 0LL;
    int32_t v6 = 0LL;
    int8_t v7 = 0LL;
    int32_t v8 = 0LL;
    int32_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int64_t v13 = arg1;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    void * v16 = ((void *)(0LL));
    uint64_t n = 0LL;
    uint64_t size = 0LL;
    int64_t v19 = arg2;
    S_1400021b0_v20_t * v20 = ((S_1400021b0_v20_t *)(0LL));
    int64_t v21 = 0LL;
    int8_t v22 = 0LL;
    void * v23 = ((void *)(0LL));
    int32_t v24 = 0LL;
    int32_t v25 = 0LL;
    void * v26 = ((void *)(0LL));
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    void * v29 = ((void *)(0LL));
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    void * v35 = ((void *)(0LL));
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;

    v1 = ((S_1400021b0_v1_t *)((v0 - 56LL)));
    v2 = (*((int32_t *)(5368742216LL)));
    v3 = v2;
    v5 = v4;
    v6 = (v3 & v3);
    v7 = (v6 != 0LL);
    if (v7) {
        v12 = ((void *)((((int64_t)(v1)) + 72LL)));
        v1->field_48 = v13;
        v14 = 1LL;
        v15 = 24LL;
        v16 = ((void *)((((int64_t)(v1)) + 64LL)));
        v1->field_40 = v5;
        v20 = ((S_1400021b0_v20_t *)(((long long (*)(long long, long long, long long, long long, long long, long long))calloc)(n, size, v15, v14, v19, v5)));
        v21 = (((int64_t)(v20)) & ((int64_t)(v20)));
        v22 = (v21 == 0LL);
        if (v22) {
            v41 = -1LL;
        } else {
            v23 = ((void *)((((int64_t)(v1)) + 64LL)));
            v24 = v1->field_40;
            v25 = v24;
            v26 = ((void *)((((int64_t)(v1)) + 72LL)));
            v27 = v1->field_48;
            v28 = v27;
            v29 = ((void *)((((int64_t)(v1)) + 40LL)));
            v1->field_28 = ((int64_t)(v20));
            v30 = 5368742240LL;
            *((int32_t *)(((int64_t)(v20)))) = v25;
            v31 = (((int64_t)(v20)) + 8LL);
            v20->field_8 = v28;
            v32 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(n, size, v15, v30, v28, v25);
            v33 = (*((int64_t *)(5368742208LL)));
            v34 = v33;
            v35 = ((void *)((((int64_t)(v1)) + 40LL)));
            v36 = v1->field_28;
            v37 = v36;
            v38 = 5368742240LL;
            v39 = (v37 + 16LL);
            *((int64_t *)(v39)) = v34;
            *((int64_t *)(5368742208LL)) = v37;
            v40 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(n, size, v34, v38, v28, v25);
L0:;
            /* phi v8 <- (bb0: v3) (bb8: v40) */
            v9 = (v8 ^ v8);
        }
    } else {
        goto L0;
    }
    /* phi v10 <- (bb1: v9) (bb9: v41) */
    v11 = (((int64_t)(v1)) + 56LL);
    return v10;
}

/* dac-recovered function */
/* address: 0x140002240 */
/* end: 0x1400022d0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 8 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: true */
/* convention: ms-x64 (score 0.95) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 1 */
/* struct_layouts: pointer=1 stack=0 */
/* switch_tables: 0 */
int64_t ___w64_mingwthr_remove_key_dtor(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int32_t v2 = 0LL;
    int32_t v3 = 0LL;
    int32_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int64_t v7 = 0LL;
    void * v8 = ((void *)(0LL));
    int64_t v9 = arg0;
    int64_t v10 = 0LL;
    int64_t p = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = arg1;
    int64_t v14 = arg2;
    int64_t v15 = arg3;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int8_t v20 = 0LL;
    void * v21 = ((void *)(0LL));
    int32_t v22 = 0LL;
    int32_t v23 = 0LL;
    int64_t v24 = 0LL;
    S_140002240_v25_t * v25 = ((S_140002240_v25_t *)(0LL));
    int64_t v26 = 0LL;
    int32_t v27 = 0LL;
    int32_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int8_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int8_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;

    v1 = (v0 - 40LL);
    v2 = (*((int32_t *)(5368742216LL)));
    v3 = v2;
    v4 = (v3 & v3);
    v5 = (v4 != 0LL);
    if (v5) {
        v8 = ((void *)((v1 + 48LL)));
        *((int32_t *)(((int64_t)(v8)))) = v9;
        v10 = 5368742240LL;
        v16 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(p, v12, v13, v10, v14, v15);
        v17 = (*((int64_t *)(5368742208LL)));
        v18 = v17;
        v19 = (v18 & v18);
        v20 = (v19 == 0LL);
        if (v20) {
        } else {
            v21 = ((void *)((v1 + 48LL)));
            v22 = (*((int32_t *)(((int64_t)(v21)))));
            v23 = v22;
            v24 = (v14 ^ v14);
            while (1) {
                /* phi v25 <- (bb5: v18) (bb8: v35) */
                /* phi v26 <- (bb5: v24) (bb8: v32) */
                v27 = (*((int32_t *)(((int64_t)(v25)))));
                v28 = v27;
                v29 = (((int64_t)(v25)) + 16LL);
                v30 = v25->field_10;
                v31 = v30;
                /* structurally unreachable: block 9 */
                __builtin_unreachable();
            }
        }
        /* phi v40 <- (bb4: v13) (bb7: v23) (bb12: v23) */
        /* phi v41 <- (bb4: v14) (bb7: v32) (bb12: v26) */
        v42 = 5368742240LL;
        v43 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(p, v12, v40, v42, v41, v15);
        v44 = (v43 ^ v43);
        v45 = (v1 + 40LL);
        return v44;
    } else {
        v6 = (v3 ^ v3);
        v7 = (v1 + 40LL);
        return v6;
    }
}

/* dac-recovered function */
/* address: 0x1400022d0 */
/* end: 0x1400023e0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 4 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.85) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 1 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t __mingw_TLScallback(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = arg2;
    int8_t v3 = 0LL;
    int64_t v4 = 0LL;
    int8_t v5 = 0LL;
    int32_t v6 = 0LL;
    int32_t v7 = 0LL;
    int32_t v8 = 0LL;
    int8_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = arg0;
    int64_t v12 = arg1;
    int64_t v13 = arg4;
    int64_t v14 = arg5;
    int64_t v15 = 0LL;
    int32_t v16 = 0LL;
    int32_t v17 = 0LL;
    int32_t v18 = 0LL;
    int8_t v19 = 0LL;
    int32_t v20 = 0LL;
    int32_t v21 = 0LL;
    int8_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int8_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    void * v32 = ((void *)(0LL));
    int64_t v33 = 0LL;
    void * v34 = ((void *)(0LL));
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int8_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = arg3;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int8_t v45 = 0LL;
    int32_t v46 = 0LL;
    int32_t v47 = 0LL;
    int32_t v48 = 0LL;
    int8_t v49 = 0LL;
    int64_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int64_t v53 = 0LL;

    v1 = (v0 - 56LL);
    v3 = (v2 == 2LL);
    if (v3) {
        v51 = ((long long (*)(long long, long long, long long, long long, long long, long long))_fpreset)(v11, v12, v2, v41, v13, v14);
        v52 = 1LL;
        v53 = (v1 + 56LL);
        return v52;
    } else {
        /* structurally unreachable: block 1 */
        __builtin_unreachable();
    }
}

/* dac-recovered function */
/* address: 0x1400023e0 */
/* end: 0x1400023f0 */
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
int64_t _fpreset(void) {
    int64_t v0 = 0LL;

    (/* opaque: fninit */ 0);
    return v0;
}

/* dac-recovered function */
/* address: 0x1400023f0 */
/* end: 0x140002420 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 4 */
/* goto_count: 1 */
/* label_count: 1 */
/* irreducible: false */
/* convention: ms-x64 (score 0.47) */
/* args: rcx */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=2 stack=0 */
/* switch_tables: 0 */
int64_t _ValidateImageBase(S_1400023f0_v2_t * arg0) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    S_1400023f0_v2_t * v2 = arg0;
    int16_t v3 = 0LL;
    int8_t v4 = 0LL;
    int64_t v5 = 0LL;
    int32_t v6 = 0LL;
    int32_t v7 = 0LL;
    S_1400023f0_v8_t * v8 = ((S_1400023f0_v8_t *)(0LL));
    int32_t v9 = 0LL;
    int8_t v10 = 0LL;
    int64_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int16_t v13 = 0LL;

    v1 = (v0 ^ v0);
    v3 = (*((int16_t *)(((int64_t)(v2)))));
    v4 = (v3 != 23117LL);
    if (v4) {
        return v1;
    } else {
        v5 = (((int64_t)(v2)) + 60LL);
        v6 = v2->field_3c;
        v7 = v6;
        v8 = ((S_1400023f0_v8_t *)((((int64_t)(v2)) + v7)));
        v9 = (*((int32_t *)(((int64_t)(v8)))));
        v10 = (v9 == 17744LL);
        if (v10) {
            v11 = (v1 ^ v1);
            v12 = ((void *)((((int64_t)(v8)) + 24LL)));
            v13 = v8->field_18;
            (/* opaque: sete */ 0);
            return v11;
        } else {
L0:;
            goto L0;
        }
    }
}

/* dac-recovered function */
/* address: 0x140002420 */
/* end: 0x140002470 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 7 */
/* goto_count: 2 */
/* label_count: 2 */
/* irreducible: false */
/* convention: ms-x64 (score 0.70) */
/* args: rcx,rdx */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=2 stack=0 */
/* switch_tables: 0 */
int64_t _FindPESection(int64_t arg0, int64_t arg1) {
    int64_t v0 = arg0;
    int64_t v1 = 0LL;
    int32_t v2 = 0LL;
    int32_t v3 = 0LL;
    S_140002420_v4_t * v4 = ((S_140002420_v4_t *)(0LL));
    void * v5 = ((void *)(0LL));
    int16_t v6 = 0LL;
    int16_t v7 = 0LL;
    int16_t v8 = 0LL;
    int8_t v9 = 0LL;
    void * v10 = ((void *)(0LL));
    int16_t v11 = 0LL;
    int16_t v12 = 0LL;
    int16_t v13 = 0LL;
    int16_t v14 = 0LL;
    int16_t v15 = 0LL;
    int16_t v16 = 0LL;
    int64_t v17 = 0LL;
    int16_t v18 = 0LL;
    int16_t v19 = 0LL;
    int16_t v20 = 0LL;
    int16_t v21 = 0LL;
    int16_t v22 = 0LL;
    int16_t v23 = 0LL;
    S_140002420_v24_t * v24 = ((S_140002420_v24_t *)(0LL));
    int64_t v25 = 0LL;
    int32_t v26 = 0LL;
    int32_t v27 = 0LL;
    int32_t v28 = 0LL;
    int64_t v29 = arg1;
    int8_t v30 = 0LL;
    int64_t v31 = 0LL;
    int32_t v32 = 0LL;
    int32_t v33 = 0LL;
    int8_t v34 = 0LL;
    int16_t v35 = 0LL;
    int8_t v36 = 0LL;
    int16_t v37 = 0LL;
    int16_t v38 = 0LL;
    int64_t v39 = 0LL;

    v1 = (v0 + 60LL);
    v2 = (*((int32_t *)(v1)));
    v3 = v2;
    v4 = ((S_140002420_v4_t *)((v3 + v0)));
    v5 = ((void *)((((int64_t)(v4)) + 6LL)));
    v6 = v4->field_6;
    v7 = v6;
    v8 = (v7 & v7);
    v9 = (v8 == 0LL);
    if (v9) {
L0:;
        /* phi v37 <- (bb0: v4) (bb4: v35) */
        v38 = (v37 ^ v37);
    } else {
        v10 = ((void *)((((int64_t)(v4)) + 20LL)));
        v11 = v4->field_14;
        v12 = v11;
        v13 = (v7 - 1LL);
        v14 = (v13 * 4LL);
        v15 = (v13 + v14);
        v16 = v15;
        v17 = (((int64_t)(v4)) + 24LL);
        v18 = (v17 + v12);
        v19 = v18;
        v20 = (v19 + 40LL);
        v21 = (v16 * 8LL);
        v22 = (v20 + v21);
        v23 = v22;
        while (1) {
            /* phi v24 <- (bb1: v19) (bb4: v35) */
            v25 = (((int64_t)(v24)) + 12LL);
            v26 = v24->field_c;
            v27 = v26;
            v28 = v27;
            v30 = (v29 < v27);
            if (v30) {
L1:;
                v35 = (((int64_t)(v24)) + 40LL);
                v36 = (v35 != v23);
                if (v36) {
                    continue;
                } else {
                    goto L0;
                }
            } else {
                v31 = (((int64_t)(v24)) + 8LL);
                v32 = v24->field_8;
                v33 = (v28 + v32);
                v34 = (v29 < v33);
                if (v34) {
                } else {
                    goto L1;
                }
            }
        }
    }
    /* phi v39 <- (bb3: v24) (bb5: v38) */
    return v39;
}

/* dac-recovered function */
/* address: 0x140002470 */
/* end: 0x140002510 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 13 */
/* goto_count: 5 */
/* label_count: 1 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.85) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 4 */
/* struct_layouts: pointer=2 stack=1 */
/* switch_tables: 0 */
int64_t _FindPESectionByName(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = arg0;
    void * v5 = ((void *)(0LL));
    int64_t v6 = arg1;
    void * v7 = ((void *)(0LL));
    int64_t v8 = 0LL;
    void * v9 = ((void *)(0LL));
    int64_t v10 = arg3;
    int64_t s = 0LL;
    int64_t v12 = arg2;
    int64_t v13 = arg4;
    int64_t v14 = arg5;
    uint64_t v15 = 0LL;
    int8_t v16 = 0LL;
    int64_t v17 = 0LL;
    S_140002470_v18_t * v18 = ((S_140002470_v18_t *)(0LL));
    int16_t v19 = 0LL;
    int8_t v20 = 0LL;
    int64_t v21 = 0LL;
    int32_t v22 = 0LL;
    int32_t v23 = 0LL;
    S_140002470_v24_t * v24 = ((S_140002470_v24_t *)(0LL));
    int32_t v25 = 0LL;
    int8_t v26 = 0LL;
    void * v27 = ((void *)(0LL));
    int16_t v28 = 0LL;
    int8_t v29 = 0LL;
    void * v30 = ((void *)(0LL));
    int16_t v31 = 0LL;
    int8_t v32 = 0LL;
    void * v33 = ((void *)(0LL));
    int16_t v34 = 0LL;
    int16_t v35 = 0LL;
    int64_t v36 = 0LL;
    void * v37 = ((void *)(0LL));
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t i = 0LL;
    int64_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    int8_t v47 = 0LL;
    void * v48 = ((void *)(0LL));
    int16_t v49 = 0LL;
    int16_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int8_t v53 = 0LL;
    int64_t v54 = 0LL;
    int64_t v55 = 0LL;
    int64_t v56 = 0LL;
    int64_t v57 = 0LL;
    void * v58 = ((void *)(0LL));
    int64_t v59 = 0LL;
    void * v60 = ((void *)(0LL));
    int64_t v61 = 0LL;
    int64_t v62 = 0LL;
    void * v63 = ((void *)(0LL));
    int64_t v64 = 0LL;
    int64_t v65 = 0LL;
    void * v66 = ((void *)(0LL));
    int64_t v67 = 0LL;
    int64_t v68 = 0LL;
    void * v69 = ((void *)(0LL));
    int64_t v70 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 8LL)));
    *((int64_t *)(((int64_t)(v5)))) = v6;
    v7 = ((void *)((((int64_t)(v5)) - 8LL)));
    *((int64_t *)(((int64_t)(v7)))) = v8;
    v9 = ((void *)((((int64_t)(v7)) - 40LL)));
    s = v10;
    v15 = ((long long (*)(long long, long long, long long, long long, long long, long long))strlen)(s, v6, v12, v10, v13, v14);
    v16 = (v15 > 8LL);
    if (v16) {
L0:;
        /* phi v54 <- (bb1: v8) (bb2: v8) (bb6: v8) (bb7: v8) (bb8: v8) (bb13: v52) */
        v55 = (v54 ^ v54);
    } else {
        v17 = (*((int64_t *)(5368726528LL)));
        v18 = ((S_140002470_v18_t *)(v17));
        v19 = (*((int16_t *)(((int64_t)(v18)))));
        v20 = (v19 == 23117LL);
        if (v20) {
            v21 = (((int64_t)(v18)) + 60LL);
            v22 = v18->field_3c;
            v23 = v22;
            v24 = ((S_140002470_v24_t *)((v23 + ((int64_t)(v18)))));
            v25 = (*((int32_t *)(((int64_t)(v24)))));
            v26 = (v25 != 17744LL);
            if (v26) {
                goto L0;
            } else {
                v27 = ((void *)((((int64_t)(v24)) + 24LL)));
                v28 = v24->field_18;
                v29 = (v28 != 523LL);
                if (v29) {
                    goto L0;
                } else {
                    v30 = ((void *)((((int64_t)(v24)) + 6LL)));
                    v31 = v24->field_6;
                    v32 = (v31 == 0LL);
                    if (v32) {
                        goto L0;
                    } else {
                        v33 = ((void *)((((int64_t)(v24)) + 20LL)));
                        v34 = v24->field_14;
                        v35 = v34;
                        v36 = (v6 ^ v6);
                        v37 = ((void *)((((int64_t)(v24)) + 24LL)));
                        v38 = (((int64_t)(v37)) + v35);
                        v39 = v38;
                        while (1) {
                            /* phi i <- (bb9: v36) (bb12: v51) */
                            /* phi v41 <- (bb9: v39) (bb12: v52) */
                            v42 = 8LL;
                            v43 = s;
                            v44 = v41;
                            v45 = ((long long (*)(long long, long long, long long, long long, long long, long long))strncmp)(s, i, v43, v44, v42, v14);
                            v46 = (v45 & v45);
                            v47 = (v46 == 0LL);
                            if (v47) {
                            } else {
                                v48 = ((void *)((((int64_t)(v24)) + 6LL)));
                                v49 = v24->field_6;
                                v50 = v49;
                                v51 = (i + 1LL);
                                v52 = (v41 + 40LL);
                                v53 = (v51 < v50);
                                if (v53) {
                                    continue;
                                } else {
                                    goto L0;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            goto L0;
        }
    }
    /* phi v56 <- (bb3: v55) (bb11: v41) */
    v57 = v56;
    v58 = ((void *)((((int64_t)(v9)) + 40LL)));
    v59 = (*((int64_t *)(((int64_t)(v58)))));
    v60 = ((void *)((((int64_t)(v58)) + 8LL)));
    v61 = v59;
    v62 = (*((int64_t *)(((int64_t)(v60)))));
    v63 = ((void *)((((int64_t)(v60)) + 8LL)));
    v64 = v62;
    v65 = (*((int64_t *)(((int64_t)(v63)))));
    v66 = ((void *)((((int64_t)(v63)) + 8LL)));
    v67 = v65;
    v68 = (*((int64_t *)(((int64_t)(v66)))));
    v69 = ((void *)((((int64_t)(v66)) + 8LL)));
    v70 = v68;
    return v57;
}

/* dac-recovered function */
/* address: 0x140002510 */
/* end: 0x140002590 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 10 */
/* goto_count: 5 */
/* label_count: 2 */
/* irreducible: false */
/* convention: ms-x64 (score 0.47) */
/* args: rcx */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=3 stack=0 */
/* switch_tables: 0 */
int64_t __mingw_GetSectionForAddress(int64_t arg0) {
    int64_t v0 = 0LL;
    S_140002510_v1_t * v1 = ((S_140002510_v1_t *)(0LL));
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int16_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int32_t v7 = 0LL;
    int32_t v8 = 0LL;
    S_140002510_v9_t * v9 = ((S_140002510_v9_t *)(0LL));
    int32_t v10 = 0LL;
    int8_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int16_t v13 = 0LL;
    int8_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int16_t v16 = 0LL;
    int16_t v17 = 0LL;
    int16_t v18 = 0LL;
    int8_t v19 = 0LL;
    void * v20 = ((void *)(0LL));
    int16_t v21 = 0LL;
    int16_t v22 = 0LL;
    int64_t v23 = arg0;
    int64_t v24 = 0LL;
    int16_t v25 = 0LL;
    int16_t v26 = 0LL;
    int16_t v27 = 0LL;
    int16_t v28 = 0LL;
    int16_t v29 = 0LL;
    void * v30 = ((void *)(0LL));
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int16_t v34 = 0LL;
    int16_t v35 = 0LL;
    int16_t v36 = 0LL;
    S_140002510_v37_t * v37 = ((S_140002510_v37_t *)(0LL));
    void * v38 = ((void *)(0LL));
    int32_t v39 = 0LL;
    int32_t v40 = 0LL;
    int32_t v41 = 0LL;
    int8_t v42 = 0LL;
    void * v43 = ((void *)(0LL));
    int32_t v44 = 0LL;
    int32_t v45 = 0LL;
    int8_t v46 = 0LL;
    int64_t v47 = 0LL;
    int8_t v48 = 0LL;
    int64_t v49 = 0LL;
    int64_t v50 = 0LL;

    v0 = (*((int64_t *)(5368726528LL)));
    v1 = ((S_140002510_v1_t *)(v0));
    v3 = (v2 ^ v2);
    v4 = (*((int16_t *)(((int64_t)(v1)))));
    v5 = (v4 != 23117LL);
    if (v5) {
L0:;
        /* phi v50 <- (bb0: v3) (bb1: v3) (bb4: v3) (bb5: v3) (bb8: v37) */
        return v50;
    } else {
        v6 = (((int64_t)(v1)) + 60LL);
        v7 = v1->field_3c;
        v8 = v7;
        v9 = ((S_140002510_v9_t *)((v8 + ((int64_t)(v1)))));
        v10 = (*((int32_t *)(((int64_t)(v9)))));
        v11 = (v10 == 17744LL);
        if (v11) {
            v12 = ((void *)((((int64_t)(v9)) + 24LL)));
            v13 = v9->field_18;
            v14 = (v13 != 523LL);
            if (v14) {
                goto L0;
            } else {
                v15 = ((void *)((((int64_t)(v9)) + 6LL)));
                v16 = v9->field_6;
                v17 = v16;
                v18 = (v17 & v17);
                v19 = (v18 == 0LL);
                if (v19) {
                    goto L0;
                } else {
                    v20 = ((void *)((((int64_t)(v9)) + 20LL)));
                    v21 = v9->field_14;
                    v22 = v21;
                    v24 = (v23 - ((int64_t)(v1)));
                    v25 = (v17 + -1LL);
                    v26 = v25;
                    v27 = (v26 * 4LL);
                    v28 = (v26 + v27);
                    v29 = v28;
                    v30 = ((void *)((((int64_t)(v9)) + 24LL)));
                    v31 = (((int64_t)(v30)) + v22);
                    v32 = v31;
                    v33 = (v32 + 40LL);
                    v34 = (v29 * 8LL);
                    v35 = (v33 + v34);
                    v36 = v35;
                    while (1) {
                        /* phi v37 <- (bb6: v32) (bb9: v47) */
                        v38 = ((void *)((((int64_t)(v37)) + 12LL)));
                        v39 = v37->field_c;
                        v40 = v39;
                        v41 = v40;
                        v42 = (v24 < v40);
                        if (v42) {
L1:;
                            v47 = (((int64_t)(v37)) + 40LL);
                            v48 = (v47 != v36);
                            if (v48) {
                                continue;
                            } else {
                                v49 = (v47 ^ v47);
                                return v49;
                            }
                        } else {
                            v43 = ((void *)((((int64_t)(v37)) + 8LL)));
                            v44 = v37->field_8;
                            v45 = (v41 + v44);
                            v46 = (v24 < v45);
                            if (v46) {
                                break;
                            } else {
                                goto L1;
                            }
                        }
                    }
                    goto L0;
                }
            }
        } else {
            goto L0;
        }
    }
}

/* dac-recovered function */
/* address: 0x140002590 */
/* end: 0x1400025d0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 5 */
/* goto_count: 2 */
/* label_count: 1 */
/* irreducible: false */
/* convention: ms-x64 (score 0.62) */
/* args: rcx */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=2 stack=0 */
/* switch_tables: 0 */
int64_t __mingw_GetSectionCount(int64_t arg0) {
    int64_t v0 = 0LL;
    S_140002590_v1_t * v1 = ((S_140002590_v1_t *)(0LL));
    int64_t v2 = arg0;
    int64_t v3 = 0LL;
    int16_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int32_t v7 = 0LL;
    int32_t v8 = 0LL;
    S_140002590_v9_t * v9 = ((S_140002590_v9_t *)(0LL));
    int32_t v10 = 0LL;
    int8_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int16_t v13 = 0LL;
    int8_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int16_t v16 = 0LL;
    int16_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;

    v0 = (*((int64_t *)(5368726528LL)));
    v1 = ((S_140002590_v1_t *)(v0));
    v3 = (v2 ^ v2);
    v4 = (*((int16_t *)(((int64_t)(v1)))));
    v5 = (v4 != 23117LL);
    if (v5) {
L0:;
        v19 = v3;
        return v19;
    } else {
        v6 = (((int64_t)(v1)) + 60LL);
        v7 = v1->field_3c;
        v8 = v7;
        v9 = ((S_140002590_v9_t *)((((int64_t)(v1)) + v8)));
        v10 = (*((int32_t *)(((int64_t)(v9)))));
        v11 = (v10 == 17744LL);
        if (v11) {
            v12 = ((void *)((((int64_t)(v9)) + 24LL)));
            v13 = v9->field_18;
            v14 = (v13 != 523LL);
            if (v14) {
                goto L0;
            } else {
                v15 = ((void *)((((int64_t)(v9)) + 6LL)));
                v16 = v9->field_6;
                v17 = v16;
                v18 = v17;
                return v18;
            }
        } else {
            goto L0;
        }
    }
}

/* dac-recovered function */
/* address: 0x1400025d0 */
/* end: 0x140002650 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 11 */
/* goto_count: 5 */
/* label_count: 2 */
/* irreducible: false */
/* convention: ms-x64 (score 0.47) */
/* args: rcx */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=2 stack=0 */
/* switch_tables: 0 */
int64_t _FindPESectionExec(int64_t arg0) {
    int64_t v0 = 0LL;
    S_1400025d0_v1_t * v1 = ((S_1400025d0_v1_t *)(0LL));
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int16_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int32_t v7 = 0LL;
    int32_t v8 = 0LL;
    S_1400025d0_v9_t * v9 = ((S_1400025d0_v9_t *)(0LL));
    int32_t v10 = 0LL;
    int8_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int16_t v13 = 0LL;
    int8_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int16_t v16 = 0LL;
    int16_t v17 = 0LL;
    int16_t v18 = 0LL;
    int8_t v19 = 0LL;
    void * v20 = ((void *)(0LL));
    int16_t v21 = 0LL;
    int16_t v22 = 0LL;
    void * v23 = ((void *)(0LL));
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int16_t v26 = 0LL;
    int16_t v27 = 0LL;
    int16_t v28 = 0LL;
    int16_t v29 = 0LL;
    int16_t v30 = 0LL;
    int64_t v31 = 0LL;
    int16_t v32 = 0LL;
    int16_t v33 = 0LL;
    int16_t v34 = 0LL;
    int64_t v35 = arg0;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    void * v38 = ((void *)(0LL));
    int8_t v39 = 0LL;
    int8_t v40 = 0LL;
    int8_t v41 = 0LL;
    int64_t v42 = 0LL;
    int8_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    int8_t v47 = 0LL;
    int64_t v48 = 0LL;
    int64_t v49 = 0LL;

    v0 = (*((int64_t *)(5368726528LL)));
    v1 = ((S_1400025d0_v1_t *)(v0));
    v3 = (v2 ^ v2);
    v4 = (*((int16_t *)(((int64_t)(v1)))));
    v5 = (v4 != 23117LL);
    if (v5) {
L0:;
        /* phi v49 <- (bb0: v3) (bb1: v3) (bb4: v3) (bb5: v3) (bb8: v36) */
        return v49;
    } else {
        v6 = (((int64_t)(v1)) + 60LL);
        v7 = v1->field_3c;
        v8 = v7;
        v9 = ((S_1400025d0_v9_t *)((v8 + ((int64_t)(v1)))));
        v10 = (*((int32_t *)(((int64_t)(v9)))));
        v11 = (v10 == 17744LL);
        if (v11) {
            v12 = ((void *)((((int64_t)(v9)) + 24LL)));
            v13 = v9->field_18;
            v14 = (v13 != 523LL);
            if (v14) {
                goto L0;
            } else {
                v15 = ((void *)((((int64_t)(v9)) + 6LL)));
                v16 = v9->field_6;
                v17 = v16;
                v18 = (v17 & v17);
                v19 = (v18 == 0LL);
                if (v19) {
                    goto L0;
                } else {
                    v20 = ((void *)((((int64_t)(v9)) + 20LL)));
                    v21 = v9->field_14;
                    v22 = v21;
                    v23 = ((void *)((((int64_t)(v9)) + 24LL)));
                    v24 = (((int64_t)(v23)) + v22);
                    v25 = v24;
                    v26 = (v17 + -1LL);
                    v27 = v26;
                    v28 = (v27 * 4LL);
                    v29 = (v27 + v28);
                    v30 = v29;
                    v31 = (v25 + 40LL);
                    v32 = (v30 * 8LL);
                    v33 = (v31 + v32);
                    v34 = v33;
                    while (1) {
                        /* phi v36 <- (bb6: v25) (bb10: v46) */
                        /* phi v37 <- (bb6: v35) (bb10: v45) */
                        v38 = ((void *)((v36 + 39LL)));
                        v39 = (*((int8_t *)(((int64_t)(v38)))));
                        v40 = (v39 & 32LL);
                        v41 = (v40 == 0LL);
                        if (v41) {
L1:;
                            /* phi v45 <- (bb7: v37) (bb9: v44) */
                            v46 = (v36 + 40LL);
                            v47 = (v34 != v46);
                            if (v47) {
                                continue;
                            } else {
                                v48 = (v46 ^ v46);
                                return v48;
                            }
                        } else {
                            v42 = (v37 & v37);
                            v43 = (v42 == 0LL);
                            if (v43) {
                                break;
                            } else {
                                v44 = (v37 - 1LL);
                                goto L1;
                            }
                        }
                    }
                    goto L0;
                }
            }
        } else {
            goto L0;
        }
    }
}

/* dac-recovered function */
/* address: 0x140002650 */
/* end: 0x140002690 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 4 */
/* goto_count: 1 */
/* label_count: 1 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.45) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=2 stack=0 */
/* switch_tables: 0 */
int64_t _GetPEImageBase(void) {
    int64_t v0 = 0LL;
    S_140002650_v1_t * v1 = ((S_140002650_v1_t *)(0LL));
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int16_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int32_t v7 = 0LL;
    int32_t v8 = 0LL;
    S_140002650_v9_t * v9 = ((S_140002650_v9_t *)(0LL));
    int32_t v10 = 0LL;
    int8_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int16_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;

    v0 = (*((int64_t *)(5368726528LL)));
    v1 = ((S_140002650_v1_t *)(v0));
    v3 = (v2 ^ v2);
    v4 = (*((int16_t *)(((int64_t)(v1)))));
    v5 = (v4 != 23117LL);
    if (v5) {
L0:;
        v15 = v3;
        return v15;
    } else {
        v6 = (((int64_t)(v1)) + 60LL);
        v7 = v1->field_3c;
        v8 = v7;
        v9 = ((S_140002650_v9_t *)((v8 + ((int64_t)(v1)))));
        v10 = (*((int32_t *)(((int64_t)(v9)))));
        v11 = (v10 == 17744LL);
        if (v11) {
            v12 = ((void *)((((int64_t)(v9)) + 24LL)));
            v13 = v9->field_18;
            (/* opaque: cmove */ 0);
            v14 = v3;
            return v14;
        } else {
            goto L0;
        }
    }
}

/* dac-recovered function */
/* address: 0x140002690 */
/* end: 0x140002720 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 11 */
/* goto_count: 4 */
/* label_count: 2 */
/* irreducible: false */
/* convention: ms-x64 (score 0.47) */
/* args: rcx */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=3 stack=0 */
/* switch_tables: 0 */
int64_t _IsNonwritableInCurrentImage(int64_t arg0) {
    int64_t v0 = 0LL;
    S_140002690_v1_t * v1 = ((S_140002690_v1_t *)(0LL));
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int16_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int32_t v7 = 0LL;
    int32_t v8 = 0LL;
    S_140002690_v9_t * v9 = ((S_140002690_v9_t *)(0LL));
    int32_t v10 = 0LL;
    int8_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int16_t v13 = 0LL;
    int8_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int16_t v16 = 0LL;
    int16_t v17 = 0LL;
    int16_t v18 = 0LL;
    int8_t v19 = 0LL;
    void * v20 = ((void *)(0LL));
    int16_t v21 = 0LL;
    int16_t v22 = 0LL;
    int64_t v23 = arg0;
    int64_t v24 = 0LL;
    int16_t v25 = 0LL;
    int16_t v26 = 0LL;
    int16_t v27 = 0LL;
    int16_t v28 = 0LL;
    int16_t v29 = 0LL;
    void * v30 = ((void *)(0LL));
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int16_t v34 = 0LL;
    int16_t v35 = 0LL;
    int16_t v36 = 0LL;
    S_140002690_v37_t * v37 = ((S_140002690_v37_t *)(0LL));
    void * v38 = ((void *)(0LL));
    int32_t v39 = 0LL;
    int32_t v40 = 0LL;
    int32_t v41 = 0LL;
    int8_t v42 = 0LL;
    void * v43 = ((void *)(0LL));
    int32_t v44 = 0LL;
    int32_t v45 = 0LL;
    int8_t v46 = 0LL;
    void * v47 = ((void *)(0LL));
    int32_t v48 = 0LL;
    int32_t v49 = 0LL;
    int32_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int8_t v53 = 0LL;
    int64_t v54 = 0LL;

    v0 = (*((int64_t *)(5368726528LL)));
    v1 = ((S_140002690_v1_t *)(v0));
    v3 = (v2 ^ v2);
    v4 = (*((int16_t *)(((int64_t)(v1)))));
    v5 = (v4 != 23117LL);
    if (v5) {
        return v3;
    } else {
        v6 = (((int64_t)(v1)) + 60LL);
        v7 = v1->field_3c;
        v8 = v7;
        v9 = ((S_140002690_v9_t *)((v8 + ((int64_t)(v1)))));
        v10 = (*((int32_t *)(((int64_t)(v9)))));
        v11 = (v10 == 17744LL);
        if (v11) {
            v12 = ((void *)((((int64_t)(v9)) + 24LL)));
            v13 = v9->field_18;
            v14 = (v13 != 523LL);
            if (v14) {
L0:;
                goto L0;
            } else {
                v15 = ((void *)((((int64_t)(v9)) + 6LL)));
                v16 = v9->field_6;
                v17 = v16;
                v18 = (v17 & v17);
                v19 = (v18 == 0LL);
                if (v19) {
                    goto L0;
                } else {
                    v20 = ((void *)((((int64_t)(v9)) + 20LL)));
                    v21 = v9->field_14;
                    v22 = v21;
                    v24 = (v23 - ((int64_t)(v1)));
                    v25 = (v17 + -1LL);
                    v26 = v25;
                    v27 = (v26 * 4LL);
                    v28 = (v26 + v27);
                    v29 = v28;
                    v30 = ((void *)((((int64_t)(v9)) + 24LL)));
                    v31 = (((int64_t)(v30)) + v22);
                    v32 = v31;
                    v33 = (v32 + 40LL);
                    v34 = (v29 * 8LL);
                    v35 = (v33 + v34);
                    v36 = v35;
                    while (1) {
                        /* phi v37 <- (bb6: v32) (bb9: v52) */
                        v38 = ((void *)((((int64_t)(v37)) + 12LL)));
                        v39 = v37->field_c;
                        v40 = v39;
                        v41 = v40;
                        v42 = (v24 < v40);
                        if (v42) {
L1:;
                            v52 = (((int64_t)(v37)) + 40LL);
                            v53 = (v36 != v52);
                            if (v53) {
                                continue;
                            } else {
                                break;
                            }
                        } else {
                            v43 = ((void *)((((int64_t)(v37)) + 8LL)));
                            v44 = v37->field_8;
                            v45 = (v41 + v44);
                            v46 = (v24 < v45);
                            if (v46) {
                                v47 = ((void *)((((int64_t)(v37)) + 36LL)));
                                v48 = v37->field_24;
                                v49 = v48;
                                v50 = ~(v49);
                                v51 = (v50 >> 31LL);
                                return v51;
                            } else {
                                goto L1;
                            }
                        }
                    }
                    v54 = (v52 ^ v52);
                    return v54;
                }
            }
        } else {
            goto L0;
        }
    }
}

/* dac-recovered function */
/* address: 0x140002720 */
/* end: 0x1400027e0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 17 */
/* goto_count: 6 */
/* label_count: 3 */
/* irreducible: true */
/* convention: ms-x64 (score 0.52) */
/* args: rcx */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=4 stack=0 */
/* switch_tables: 0 */
int64_t __mingw_enum_import_library_names(int64_t arg0) {
    int64_t v0 = 0LL;
    S_140002720_v1_t * v1 = ((S_140002720_v1_t *)(0LL));
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int16_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int32_t v7 = 0LL;
    int32_t v8 = 0LL;
    S_140002720_v9_t * v9 = ((S_140002720_v9_t *)(0LL));
    int32_t v10 = 0LL;
    int8_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int16_t v13 = 0LL;
    int8_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int32_t v16 = 0LL;
    int32_t v17 = 0LL;
    int32_t v18 = 0LL;
    int8_t v19 = 0LL;
    void * v20 = ((void *)(0LL));
    int16_t v21 = 0LL;
    int16_t v22 = 0LL;
    int16_t v23 = 0LL;
    int8_t v24 = 0LL;
    void * v25 = ((void *)(0LL));
    int16_t v26 = 0LL;
    int16_t v27 = 0LL;
    void * v28 = ((void *)(0LL));
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int16_t v31 = 0LL;
    int16_t v32 = 0LL;
    int16_t v33 = 0LL;
    int16_t v34 = 0LL;
    int16_t v35 = 0LL;
    int64_t v36 = 0LL;
    int16_t v37 = 0LL;
    int16_t v38 = 0LL;
    int16_t v39 = 0LL;
    S_140002720_v40_t * v40 = ((S_140002720_v40_t *)(0LL));
    void * v41 = ((void *)(0LL));
    int32_t v42 = 0LL;
    int32_t v43 = 0LL;
    int32_t v44 = 0LL;
    int8_t v45 = 0LL;
    void * v46 = ((void *)(0LL));
    int32_t v47 = 0LL;
    int32_t v48 = 0LL;
    int8_t v49 = 0LL;
    int64_t v50 = 0LL;
    int64_t v51 = arg0;
    S_140002720_v52_t * v52 = ((S_140002720_v52_t *)(0LL));
    int64_t v53 = 0LL;
    void * v54 = ((void *)(0LL));
    int32_t v55 = 0LL;
    int32_t v56 = 0LL;
    int32_t v57 = 0LL;
    int8_t v58 = 0LL;
    void * v59 = ((void *)(0LL));
    int32_t v60 = 0LL;
    int32_t v61 = 0LL;
    int32_t v62 = 0LL;
    int8_t v63 = 0LL;
    int64_t v64 = 0LL;
    int8_t v65 = 0LL;
    int64_t v66 = 0LL;
    int64_t v67 = 0LL;
    void * v68 = ((void *)(0LL));
    int32_t v69 = 0LL;
    int32_t v70 = 0LL;
    int64_t v71 = 0LL;
    int64_t v72 = 0LL;
    int32_t v73 = 0LL;
    int64_t v74 = 0LL;
    int8_t v75 = 0LL;
    int32_t v76 = 0LL;
    int32_t v77 = 0LL;
    int64_t v78 = 0LL;
    int64_t v79 = 0LL;

    v0 = (*((int64_t *)(5368726528LL)));
    v1 = ((S_140002720_v1_t *)(v0));
    v3 = (v2 ^ v2);
    v4 = (*((int16_t *)(((int64_t)(v1)))));
    v5 = (v4 != 23117LL);
    if (v5) {
L0:;
        v79 = v3;
        return v79;
    } else {
        v6 = (((int64_t)(v1)) + 60LL);
        v7 = v1->field_3c;
        v8 = v7;
        v9 = ((S_140002720_v9_t *)((v8 + ((int64_t)(v1)))));
        v10 = (*((int32_t *)(((int64_t)(v9)))));
        v11 = (v10 == 17744LL);
        if (v11) {
            v12 = ((void *)((((int64_t)(v9)) + 24LL)));
            v13 = v9->field_18;
            v14 = (v13 != 523LL);
            if (v14) {
                goto L0;
            } else {
                v15 = ((void *)((((int64_t)(v9)) + 144LL)));
                v16 = v9->field_90;
                v17 = v16;
                v18 = (v17 & v17);
                v19 = (v18 == 0LL);
                if (v19) {
                    goto L0;
                } else {
                    v20 = ((void *)((((int64_t)(v9)) + 6LL)));
                    v21 = v9->field_6;
                    v22 = v21;
                    v23 = (v22 & v22);
                    v24 = (v23 == 0LL);
                    if (v24) {
                        goto L0;
                    } else {
                        v25 = ((void *)((((int64_t)(v9)) + 20LL)));
                        v26 = v9->field_14;
                        v27 = v26;
                        v28 = ((void *)((((int64_t)(v9)) + 24LL)));
                        v29 = (((int64_t)(v28)) + v27);
                        v30 = v29;
                        v31 = (v22 + -1LL);
                        v32 = v31;
                        v33 = (v32 * 4LL);
                        v34 = (v32 + v33);
                        v35 = v34;
                        v36 = (v30 + 40LL);
                        v37 = (v35 * 8LL);
                        v38 = (v36 + v37);
                        v39 = v38;
                        while (1) {
                            /* phi v40 <- (bb7: v30) (bb10: v74) */
                            v41 = ((void *)((((int64_t)(v40)) + 12LL)));
                            v42 = v40->field_c;
                            v43 = v42;
                            v44 = v43;
                            v45 = (v17 < v43);
                            if (v45) {
L2:;
                                /* phi v73 <- (bb8: v44) (bb9: v48) */
                                v74 = (((int64_t)(v40)) + 40LL);
                                v75 = (v39 != v74);
                                if (v75) {
                                    continue;
                                } else {
                                    break;
                                }
                            } else {
                                v46 = ((void *)((((int64_t)(v40)) + 8LL)));
                                v47 = v40->field_8;
                                v48 = (v44 + v47);
                                v49 = (v17 < v48);
                                if (v49) {
                                    v50 = (v17 + ((int64_t)(v1)));
                                    while (1) {
                                        /* phi v52 <- (bb13: v50) (bb15: v67) */
                                        /* phi v53 <- (bb13: v51) (bb15: v66) */
                                        v54 = ((void *)((((int64_t)(v52)) + 4LL)));
                                        v55 = v52->field_4;
                                        v56 = v55;
                                        v57 = (v56 & v56);
                                        v58 = (v57 != 0LL);
                                        if (v58) {
L1:;
                                            v64 = (v53 & v53);
                                            v65 = (v64 > 0LL);
                                            if (v65) {
                                                v66 = (v53 - 1LL);
                                                v67 = (((int64_t)(v52)) + 20LL);
                                                continue;
                                            } else {
                                                v68 = ((void *)((((int64_t)(v52)) + 12LL)));
                                                v69 = v52->field_c;
                                                v70 = v69;
                                                v71 = (v70 + ((int64_t)(v1)));
                                                v72 = v71;
                                                return v72;
                                            }
                                        } else {
                                            v59 = ((void *)((((int64_t)(v52)) + 12LL)));
                                            v60 = v52->field_c;
                                            v61 = v60;
                                            v62 = (v61 & v61);
                                            v63 = (v62 == 0LL);
                                            if (v63) {
                                                break;
                                            } else {
                                                goto L1;
                                            }
                                        }
                                    }
                                    break;
                                } else {
                                    goto L2;
                                }
                            }
                        }
                        /* phi v76 <- (bb10: v73) (bb17: v56) */
                        v77 = (v76 ^ v76);
                        v78 = v77;
                        return v78;
                    }
                }
            }
        } else {
            goto L0;
        }
    }
}

/* dac-recovered function */
/* address: 0x1400027e0 */
/* end: 0x140002820 */
/* confidence: 0.85 (Derived) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.47) */
/* args: rcx */
/* return_reg: rax */
/* stack_locals: 2 */
/* struct_layouts: pointer=1 stack=1 */
/* switch_tables: 0 */
int64_t fn_1400027e0(int64_t arg0) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = arg0;
    S_1400027e0_v3_t * v3 = ((S_1400027e0_v3_t *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    void * v6 = ((void *)(0LL));
    void * v7 = ((void *)(0LL));
    int64_t v8 = 0LL;
    void * v9 = ((void *)(0LL));
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int8_t v13 = 0LL;
    void * v14 = ((void *)(0LL));
    int64_t v15 = 0LL;
    void * v16 = ((void *)(0LL));
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    void * v20 = ((void *)(0LL));
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    void * v23 = ((void *)(0LL));
    int64_t v24 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((S_1400027e0_v3_t *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) + 24LL)));
    v6 = ((void *)(((int64_t)(v5))));
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002820 */
/* end: 0x140002860 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 3 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.85) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 4 */
/* struct_layouts: pointer=1 stack=1 */
/* switch_tables: 0 */
int64_t vfprintf(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = arg0;
    void * v3 = ((void *)(0LL));
    int64_t v4 = arg1;
    void * v5 = ((void *)(0LL));
    int64_t v6 = 0LL;
    S_140002820_v7_t * v7 = ((S_140002820_v7_t *)(0LL));
    int64_t v8 = arg3;
    int64_t v9 = 0LL;
    int64_t v10 = arg2;
    int64_t v11 = 0LL;
    int64_t v12 = arg4;
    int64_t v13 = 0LL;
    int64_t v14 = arg5;
    void * v15 = ((void *)(0LL));
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    void * v21 = ((void *)(0LL));
    int64_t v22 = 0LL;
    void * v23 = ((void *)(0LL));
    int64_t v24 = 0LL;
    void * v25 = ((void *)(0LL));
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    void * v28 = ((void *)(0LL));
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    void * v31 = ((void *)(0LL));
    int64_t v32 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 8LL)));
    *((int64_t *)(((int64_t)(v5)))) = v6;
    v7 = ((S_140002820_v7_t *)((((int64_t)(v5)) - 48LL)));
    v9 = v8;
    v11 = v10;
    v13 = v12;
    v15 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__local_stdio_printf_options)(v13, v11, v10, v8, v12, v14)));
    v16 = (v14 ^ v14);
    v17 = v11;
    v18 = v9;
    v19 = (*((int64_t *)(((int64_t)(v15)))));
    v20 = v19;
    v21 = ((void *)((((int64_t)(v7)) + 32LL)));
    v7->field_20 = v13;
    v22 = ((long long (*)(long long, long long, long long, long long, long long, long long))__stdio_common_vfprintf)(v13, v11, v18, v20, v17, v16);
    v23 = ((void *)((((int64_t)(v7)) + 48LL)));
    v24 = v7->field_30;
    v25 = ((void *)((((int64_t)(v23)) + 8LL)));
    v26 = v24;
    v27 = (*((int64_t *)(((int64_t)(v25)))));
    v28 = ((void *)((((int64_t)(v25)) + 8LL)));
    v29 = v27;
    v30 = (*((int64_t *)(((int64_t)(v28)))));
    v31 = ((void *)((((int64_t)(v28)) + 8LL)));
    v32 = v30;
    return v22;
}

/* dac-recovered function */
/* address: 0x140002860 */
/* end: 0x1400028b0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 3 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.95) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 7 */
/* struct_layouts: pointer=1 stack=1 */
/* switch_tables: 0 */
int64_t fprintf(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    int64_t v6 = 0LL;
    S_140002860_v7_t * v7 = ((S_140002860_v7_t *)(0LL));
    void * v8 = ((void *)(0LL));
    int64_t v9 = 0LL;
    int64_t v10 = arg0;
    int64_t v11 = 0LL;
    int64_t v12 = arg1;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = arg2;
    void * v16 = ((void *)(0LL));
    int64_t v17 = arg3;
    void * v18 = ((void *)(0LL));
    void * v19 = ((void *)(0LL));
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    void * v25 = ((void *)(0LL));
    int64_t v26 = 0LL;
    void * v27 = ((void *)(0LL));
    int64_t v28 = 0LL;
    void * v29 = ((void *)(0LL));
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    void * v32 = ((void *)(0LL));
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    void * v35 = ((void *)(0LL));
    int64_t v36 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 8LL)));
    *((int64_t *)(((int64_t)(v5)))) = v6;
    v7 = ((S_140002860_v7_t *)((((int64_t)(v5)) - 64LL)));
    v8 = ((void *)((((int64_t)(v7)) + 112LL)));
    v9 = ((int64_t)(v8));
    v11 = v10;
    v13 = v12;
    v7->field_70 = v15;
    v16 = ((void *)((((int64_t)(v7)) + 120LL)));
    v7->field_78 = v17;
    v18 = ((void *)((((int64_t)(v7)) + 56LL)));
    v7->field_38 = v9;
    v19 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__local_stdio_printf_options)(v9, v13, v12, v10, v15, v17)));
    v20 = (v17 ^ v17);
    v21 = v13;
    v22 = v11;
    v23 = (*((int64_t *)(((int64_t)(v19)))));
    v24 = v23;
    v25 = ((void *)((((int64_t)(v7)) + 32LL)));
    v7->field_20 = v9;
    v26 = ((long long (*)(long long, long long, long long, long long, long long, long long))__stdio_common_vfprintf)(v9, v13, v22, v24, v21, v20);
    v27 = ((void *)((((int64_t)(v7)) + 64LL)));
    v28 = v7->field_40;
    v29 = ((void *)((((int64_t)(v27)) + 8LL)));
    v30 = v28;
    v31 = (*((int64_t *)(((int64_t)(v29)))));
    v32 = ((void *)((((int64_t)(v29)) + 8LL)));
    v33 = v31;
    v34 = (*((int64_t *)(((int64_t)(v32)))));
    v35 = ((void *)((((int64_t)(v32)) + 8LL)));
    v36 = v34;
    return v26;
}

/* dac-recovered function */
/* address: 0x1400028b0 */
/* end: 0x1400028c0 */
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
int64_t __local_stdio_printf_options(void) {
    int64_t v0 = 0LL;

    v0 = 5368721504LL;
    return v0;
}

/* dac-recovered function */
/* address: 0x1400028c0 */
/* end: 0x1400028d0 */
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
int64_t __p___initenv(void) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;

    v0 = (*((int64_t *)(5368726608LL)));
    v1 = v0;
    v2 = (*((int64_t *)(v1)));
    v3 = v2;
    return v3;
}

/* dac-recovered function */
/* address: 0x1400028d0 */
/* end: 0x140002900 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 4 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.70) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: none */
/* stack_locals: 1 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void _amsg_exit(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = arg3;
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;
    int64_t v7 = arg0;
    int64_t v8 = arg1;
    int64_t v9 = arg2;
    int64_t v10 = arg4;
    int64_t v11 = arg5;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 32LL)));
    v5 = v4;
    v6 = 2LL;
    v12 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v7, v8, v9, v6, v10, v11);
    v13 = v5;
    v14 = 5368726448LL;
    v15 = v12;
    v16 = ((long long (*)(long long, long long, long long, long long, long long, long long))fprintf)(v7, v8, v14, v15, v13, v11);
    v17 = 255LL;
    v18 = ((long long (*)(long long, long long, long long, long long, long long, long long))_exit)(v7, v8, v14, v17, v13, v11);
    /* structurally unreachable: block 3 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002900 */
/* end: 0x140002960 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 7 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.90) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 5 */
/* struct_layouts: pointer=1 stack=1 */
/* switch_tables: 0 */
int64_t __getmainargs(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = arg0;
    void * v5 = ((void *)(0LL));
    int64_t v6 = arg1;
    void * v7 = ((void *)(0LL));
    int64_t v8 = 0LL;
    S_140002900_v9_t * v9 = ((S_140002900_v9_t *)(0LL));
    int64_t v10 = arg5;
    int64_t v11 = 0LL;
    int64_t v12 = arg2;
    int64_t v13 = 0LL;
    int64_t v14 = arg4;
    int64_t v15 = 0LL;
    int64_t v16 = arg3;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    void * v22 = ((void *)(0LL));
    int32_t v23 = 0LL;
    int32_t v24 = 0LL;
    void * v25 = ((void *)(0LL));
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    void * v28 = ((void *)(0LL));
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    void * v31 = ((void *)(0LL));
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int32_t v34 = 0LL;
    int32_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    void * v38 = ((void *)(0LL));
    int64_t v39 = 0LL;
    void * v40 = ((void *)(0LL));
    int64_t v41 = 0LL;
    int64_t v42 = 0LL;
    void * v43 = ((void *)(0LL));
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    void * v46 = ((void *)(0LL));
    int64_t v47 = 0LL;
    int64_t v48 = 0LL;
    void * v49 = ((void *)(0LL));
    int64_t v50 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 8LL)));
    *((int64_t *)(((int64_t)(v5)))) = v6;
    v7 = ((void *)((((int64_t)(v5)) - 8LL)));
    *((int64_t *)(((int64_t)(v7)))) = v8;
    v9 = ((S_140002900_v9_t *)((((int64_t)(v7)) - 40LL)));
    v11 = v10;
    v13 = v12;
    v15 = v14;
    v17 = v16;
    v18 = ((long long (*)(long long, long long, long long, long long, long long, long long))_initialize_narrow_environment)(v17, v13, v12, v16, v14, v10);
    v19 = 1LL;
    v20 = (v19 - -1LL);
    v21 = ((long long (*)(long long, long long, long long, long long, long long, long long))_configure_narrow_argv)(v17, v13, v12, v20, v14, v10);
    v22 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__p___argc)(v17, v13, v12, v20, v14, v10)));
    v23 = (*((int32_t *)(((int64_t)(v22)))));
    v24 = v23;
    *((int32_t *)(v17)) = v24;
    v25 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__p___argv)(v17, v13, v12, v20, v14, v10)));
    v26 = (*((int64_t *)(((int64_t)(v25)))));
    v27 = v26;
    *((int64_t *)(v13)) = v27;
    v28 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__p__environ)(v17, v13, v12, v20, v14, v10)));
    v29 = (*((int64_t *)(((int64_t)(v28)))));
    v30 = v29;
    *((int64_t *)(v15)) = v30;
    v31 = ((void *)((((int64_t)(v9)) + 112LL)));
    v32 = v9->field_70;
    v33 = v32;
    v34 = (*((int32_t *)(v33)));
    v35 = v34;
    v36 = ((long long (*)(long long, long long, long long, long long, long long, long long))_set_new_mode)(v17, v13, v12, v35, v14, v10);
    v37 = (v36 ^ v36);
    v38 = ((void *)((((int64_t)(v9)) + 40LL)));
    v39 = v9->field_28;
    v40 = ((void *)((((int64_t)(v38)) + 8LL)));
    v41 = v39;
    v42 = (*((int64_t *)(((int64_t)(v40)))));
    v43 = ((void *)((((int64_t)(v40)) + 8LL)));
    v44 = v42;
    v45 = (*((int64_t *)(((int64_t)(v43)))));
    v46 = ((void *)((((int64_t)(v43)) + 8LL)));
    v47 = v45;
    v48 = (*((int64_t *)(((int64_t)(v46)))));
    v49 = ((void *)((((int64_t)(v46)) + 8LL)));
    v50 = v48;
    return v37;
}

/* dac-recovered function */
/* address: 0x140002960 */
/* end: 0x140002968 */
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
void strlen(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002968 */
/* end: 0x140002970 */
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
void strncmp(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002970 */
/* end: 0x140002978 */
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
void __acrt_iob_func(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002978 */
/* end: 0x140002980 */
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
void __p__commode(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002980 */
/* end: 0x140002988 */
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
void __p__fmode(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002988 */
/* end: 0x140002990 */
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
void __stdio_common_vfprintf(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002990 */
/* end: 0x140002998 */
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
void fflush(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002998 */
/* end: 0x1400029a0 */
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
void setvbuf(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1400029a0 */
/* end: 0x1400029a8 */
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
void __set_app_type(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1400029a8 */
/* end: 0x1400029b0 */
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
void __p___argc(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1400029b0 */
/* end: 0x1400029b8 */
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
void __p___argv(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1400029b8 */
/* end: 0x1400029c0 */
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
void _cexit(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1400029c0 */
/* end: 0x1400029c8 */
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
void _configure_narrow_argv(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1400029c8 */
/* end: 0x1400029d0 */
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
void _crt_atexit(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1400029d0 */
/* end: 0x1400029d8 */
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
void _exit(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1400029d8 */
/* end: 0x1400029e0 */
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
void _initialize_narrow_environment(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1400029e0 */
/* end: 0x1400029e8 */
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
void _initterm(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1400029e8 */
/* end: 0x1400029f0 */
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
void _initterm_e(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1400029f0 */
/* end: 0x1400029f8 */
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
void _set_invalid_parameter_handler(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x1400029f8 */
/* end: 0x140002a00 */
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
void abort(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002a00 */
/* end: 0x140002a08 */
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
void exit(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002a08 */
/* end: 0x140002a18 */
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
void signal(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002a18 */
/* end: 0x140002a20 */
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
void memcpy(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002a20 */
/* end: 0x140002a28 */
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
void __setusermatherr(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002a28 */
/* end: 0x140002a30 */
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
void _configthreadlocale(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002a30 */
/* end: 0x140002a38 */
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
void _set_new_mode(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002a38 */
/* end: 0x140002a40 */
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
void calloc(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002a40 */
/* end: 0x140002a48 */
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
void free(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002a48 */
/* end: 0x140002a50 */
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
void malloc(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002a50 */
/* end: 0x140002a60 */
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
void __p__environ(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}

/* dac-recovered function */
/* address: 0x140002a60 */
/* end: 0x140002ac0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 4 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: sysv-amd64 (score 0.85) */
/* args: rdi,rsi,rdx,rcx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 3 */
/* struct_layouts: pointer=1 stack=0 */
/* switch_tables: 0 */
int64_t main(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    S_140002a60_v1_t * v1 = ((S_140002a60_v1_t *)(0LL));
    int64_t v2 = arg0;
    int64_t v3 = arg1;
    int64_t v4 = arg2;
    int64_t v5 = arg3;
    int64_t v6 = arg4;
    int64_t v7 = arg5;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    void * v13 = ((void *)(0LL));
    int64_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    void * v19 = ((void *)(0LL));
    int64_t v20 = 0LL;
    void * v21 = ((void *)(0LL));
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;

    v1 = ((S_140002a60_v1_t *)((v0 - 88LL)));
    v8 = ((long long (*)(long long, long long, long long, long long, long long, long long))__main)(v2, v3, v4, v5, v6, v7);
    v9 = -11LL;
    v10 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v2, v3, v4, v9, v6, v7);
    (/* opaque: movaps */ 0);
    v11 = (v4 ^ v4);
    v12 = (v9 ^ v9);
    v13 = ((void *)((((int64_t)(v1)) + 56LL)));
    v1->field_38 = v11;
    v15 = ((void *)(((int64_t)(v13))));
    v16 = (((int64_t)(v1)) + 61LL);
    v17 = v16;
    v18 = 18LL;
    v19 = ((void *)((((int64_t)(v1)) + 32LL)));
    v1->field_20 = v12;
    v20 = v10;
    (/* opaque: movups */ 0);
    v21 = ((void *)((((int64_t)(v1)) + 76LL)));
    v1->field_4c = 673104LL;
    v22 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v2, v3, v17, v20, v18, ((int64_t)(v15)));
    v23 = 42LL;
    v24 = (((int64_t)(v1)) + 88LL);
    return v23;
}

/* dac-recovered function */
/* address: 0x140002ac0 */
/* end: 0x140002ad0 */
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
void register_frame_ctor(void) {
    /* structurally unreachable: block 0 */
    __builtin_unreachable();
}
