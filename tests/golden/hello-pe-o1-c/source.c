/* dac --target c -O1 reconstruction
   input: tests/fixtures/hello-x86_64.exe
   arch:  x86-64 */
#include <stdint.h>
#include <stddef.h>

/* dac-recovered struct */
/* base: v145 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_140001020_v145_t;

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
/* base: v47 */
/* total_size: 8 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    uint8_t __pad_0_8[8];
    int32_t field_8;
    int32_t field_c;
} S_140001890_v47_t;

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
/* base: v67 */
/* total_size: 12 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    int32_t field_4;
    int32_t field_8;
} S_140001a00_v67_t;

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
/* base: v5 */
/* total_size: 5 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int32_t field_0;
    int8_t field_4;
} S_140001f80_v5_t;

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
/* base: v17 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_140002470_v17_t;

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
/* base: v0 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_140002510_v0_t;

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
/* base: v0 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_140002590_v0_t;

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
/* base: v0 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_1400025d0_v0_t;

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
/* base: v0 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_140002650_v0_t;

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
/* base: v0 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_140002690_v0_t;

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
/* base: v0 */
/* total_size: 64 bytes */
/* confidence: 0.65 (Derived) */
typedef struct __attribute__((packed)) {
    int16_t field_0;
    uint8_t __pad_2_3c[58];
    int32_t field_3c;
} S_140002720_v0_t;

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
/* convention: ms-x64 (score 0.40) */
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

    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140001020 */
/* end: 0x140001420 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 45 */
/* goto_count: 2 */
/* label_count: 2 */
/* irreducible: true */
/* convention: ms-x64 (score 0.85) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 11 */
/* struct_layouts: pointer=3 stack=1 */
/* switch_tables: 0 */
int64_t __tmainCRTStartup(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
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
    int64_t v12 = 0LL;
    void * v13 = ((void *)(0LL));
    int64_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int64_t v16 = 0LL;
    S_140001020_v17_t * v17 = ((S_140001020_v17_t *)(0LL));
    int64_t v18 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v23 = 0LL;
    int64_t v25 = 0LL;
    int64_t v27 = arg0;
    int64_t v29 = 0LL;
    int8_t v31 = 0LL;
    int64_t v33 = arg1;
    int64_t v34 = arg2;
    int64_t v35 = arg3;
    int64_t v36 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int32_t v42 = 0LL;
    int8_t v44 = 0LL;
    int32_t v45 = 0LL;
    int8_t v48 = 0LL;
    int64_t v49 = 0LL;
    int64_t v50 = 0LL;
    int64_t v52 = 0LL;
    int64_t v53 = 0LL;
    int64_t v54 = 0LL;
    int64_t v55 = 0LL;
    int8_t v57 = 0LL;
    int64_t v58 = 0LL;
    int64_t v60 = 0LL;
    int8_t v63 = 0LL;
    int64_t v67 = 0LL;
    int64_t v68 = 0LL;
    int64_t v69 = 0LL;
    int64_t v70 = 0LL;
    void * v71 = ((void *)(0LL));
    int64_t v72 = 0LL;
    int32_t v74 = 0LL;
    int64_t v76 = 0LL;
    int32_t v78 = 0LL;
    int32_t v79 = 0LL;
    int8_t v82 = 0LL;
    int32_t v83 = 0LL;
    int8_t v86 = 0LL;
    int64_t v87 = 0LL;
    void * v88 = ((void *)(0LL));
    int64_t v89 = 0LL;
    void * v90 = ((void *)(0LL));
    int64_t v92 = 0LL;
    void * v93 = ((void *)(0LL));
    int64_t v95 = 0LL;
    void * v96 = ((void *)(0LL));
    int64_t v98 = 0LL;
    void * v99 = ((void *)(0LL));
    int64_t v101 = 0LL;
    void * v102 = ((void *)(0LL));
    int64_t v104 = 0LL;
    void * v105 = ((void *)(0LL));
    int64_t v107 = 0LL;
    void * v108 = ((void *)(0LL));
    int64_t v110 = 0LL;
    void * v113 = ((void *)(0LL));
    int64_t v114 = 0LL;
    void * v115 = ((void *)(0LL));
    int32_t v116 = 0LL;
    int64_t v120 = 0LL;
    int64_t v125 = 0LL;
    int64_t v127 = 0LL;
    int8_t v130 = 0LL;
    int64_t v131 = 0LL;
    int64_t v133 = 0LL;
    int64_t v134 = 0LL;
    int64_t v137 = 0LL;
    int64_t v138 = 0LL;
    int64_t v139 = 0LL;
    int64_t v141 = 0LL;
    int64_t v143 = 0LL;
    S_140001020_v145_t * v145 = ((S_140001020_v145_t *)(0LL));
    int16_t v147 = 0LL;
    int8_t v148 = 0LL;
    int64_t v149 = 0LL;
    int32_t v150 = 0LL;
    S_140001020_v152_t * v152 = ((S_140001020_v152_t *)(0LL));
    int32_t v153 = 0LL;
    int8_t v154 = 0LL;
    void * v155 = ((void *)(0LL));
    int16_t v156 = 0LL;
    int8_t v158 = 0LL;
    int8_t v159 = 0LL;
    void * v160 = ((void *)(0LL));
    int32_t v161 = 0LL;
    int8_t v162 = 0LL;
    void * v163 = ((void *)(0LL));
    int32_t v164 = 0LL;
    void * v167 = ((void *)(0LL));
    int32_t v168 = 0LL;
    int8_t v169 = 0LL;
    void * v170 = ((void *)(0LL));
    int32_t v171 = 0LL;
    int32_t v174 = 0LL;
    int64_t v175 = 0LL;
    int64_t v176 = 0LL;
    int64_t v177 = 0LL;
    int32_t v179 = 0LL;
    int8_t v182 = 0LL;
    int64_t v184 = 0LL;
    int64_t n = 0LL;
    int64_t v186 = 0LL;
    int64_t v187 = 0LL;
    int64_t v188 = 0LL;
    int64_t v189 = 0LL;
    int64_t v190 = 0LL;
    void * v191 = ((void *)(0LL));
    int64_t v192 = 0LL;
    int32_t v194 = 0LL;
    void * v196 = ((void *)(0LL));
    int64_t v197 = 0LL;
    int32_t v199 = 0LL;
    int64_t v201 = 0LL;
    int64_t v202 = 0LL;
    int32_t v204 = 0LL;
    int8_t v205 = 0LL;
    int64_t v206 = 0LL;
    int32_t v208 = 0LL;
    int8_t v209 = 0LL;
    int64_t v210 = 0LL;
    int64_t v212 = 0LL;
    int64_t v214 = 0LL;
    int8_t v216 = 0LL;
    int64_t v217 = 0LL;
    int32_t v222 = 0LL;
    void * v224 = ((void *)(0LL));
    int64_t v225 = 0LL;
    int32_t v227 = 0LL;
    void * v231 = ((void *)(0LL));
    int64_t v232 = 0LL;
    int32_t v233 = 0LL;
    int32_t v235 = 0LL;
    int32_t v238 = 0LL;
    void * v239 = ((void *)(0LL));
    int8_t v242 = 0LL;
    int8_t v244 = 0LL;
    int64_t v245 = 0LL;
    int64_t s = 0LL;
    void * src = ((void *)(0LL));
    uint64_t v250 = 0LL;
    int64_t v251 = 0LL;
    int64_t v252 = 0LL;
    void * v253 = ((void *)(0LL));
    void * v254 = ((void *)(0LL));
    int64_t v255 = 0LL;
    uint64_t v257 = 0LL;
    uint64_t size = 0LL;
    int64_t v261 = 0LL;
    int64_t v262 = 0LL;
    void * v263 = ((void *)(0LL));
    void * v264 = ((void *)(0LL));
    int8_t v266 = 0LL;
    int64_t v267 = 0LL;
    void * v268 = ((void *)(0LL));
    void * v269 = ((void *)(0LL));
    int64_t n_1 = 0LL;
    void * v274 = ((void *)(0LL));
    int8_t v275 = 0LL;
    void * v276 = ((void *)(0LL));
    void * v278 = ((void *)(0LL));
    int64_t v279 = 0LL;
    int64_t v281 = 0LL;
    int64_t v282 = 0LL;
    void * v283 = ((void *)(0LL));
    int64_t v284 = 0LL;
    int64_t v285 = 0LL;
    int64_t v287 = 0LL;
    int64_t v289 = 0LL;
    int64_t v290 = 0LL;
    int64_t v292 = 0LL;
    int64_t v294 = 0LL;
    int64_t v295 = 0LL;
    int64_t v296 = 0LL;
    int64_t v297 = 0LL;
    int64_t v298 = 0LL;
    int64_t v299 = 0LL;
    int64_t v301 = 0LL;
    int64_t v302 = 0LL;
    int64_t v303 = 0LL;
    int64_t v304 = 0LL;
    int64_t v305 = 0LL;
    int64_t v306 = 0LL;
    int64_t v308 = 0LL;
    int64_t v309 = 0LL;
    int64_t v310 = 0LL;
    int64_t v311 = 0LL;
    int64_t v312 = 0LL;
    int64_t v313 = 0LL;
    int64_t v314 = 0LL;
    int64_t v316 = 0LL;
    int64_t status = 0LL;
    int64_t v318 = 0LL;
    int32_t v319 = 0LL;
    int64_t v320 = 0LL;
    int64_t v321 = 0LL;
    int64_t v322 = 0LL;
    int64_t v324 = 0LL;
    int64_t v325 = 0LL;
    int64_t v326 = 0LL;
    int64_t v327 = 0LL;
    int64_t v328 = 0LL;
    int64_t v329 = 0LL;
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
    v20 = (v18 + 8LL);
    v21 = (*((int64_t *)(v20)));
    v23 = (*((int64_t *)(5368726704LL)));
    v25 = (*((int64_t *)(5368746720LL)));
    while (1) {
        /* phi v29 <- (bb0: v27) (bb3: 1000) */
        (/* opaque: cmpxchg */ 0);
        /* dac: structuring fallback */
    }
    /* phi v39 <- (bb5: 0) (bb18: 1) */
    v40 = (*((int64_t *)(5368726720LL)));
    v42 = (*((int32_t *)(v40)));
    v44 = (v42 == 1LL);
    if (v44) {
L0:;
        /* phi v325 <- (bb6: v25) (bb79: status) */
        /* phi v326 <- (bb6: v21) (bb79: v318) */
        /* phi v327 <- (bb6: v33) (bb79: v320) */
        /* phi v328 <- (bb6: v34) (bb79: v321) */
        /* phi v329 <- (bb6: v35) (bb79: v322) */
        v331 = ((long long (*)(long long, long long, long long, long long, long long, long long))_amsg_exit)(v325, v326, v327, 31LL, v328, v329);
        /* dac: structuring fallback */
    } else {
        v45 = (*((int32_t *)(v40)));
        v48 = (v45 == 0LL);
        if (v48) {
            *((int32_t *)(v40)) = 1LL;
            v120 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v25, v21, v33, 2LL, v34, v35);
            v125 = ((long long (*)(long long, long long, long long, long long, long long, long long))setvbuf)(v25, v21, 0LL, v120, 4LL, 0LL);
            v127 = ((long long (*)(long long, long long, long long, long long, long long, long long))_crt_atexit)(v25, v21, 0LL, 5368713232LL, 4LL, 0LL);
            v130 = (v127 != 0LL);
            if (v130) {
                v309 = ((long long (*)(long long, long long, long long, long long, long long, long long))abort)(v25, v127, 0LL, 5368713232LL, 4LL, 0LL);
                /* phi v310 <- (bb48: n) (bb77: v25) */
                /* phi v311 <- (bb48: v186) (bb77: v127) */
                /* phi v312 <- (bb48: v210) (bb77: 0) */
                /* phi v313 <- (bb48: v189) (bb77: 4) */
                /* phi v314 <- (bb48: v190) (bb77: 0) */
                v316 = ((long long (*)(long long, long long, long long, long long, long long, long long))_amsg_exit)(v310, v311, v312, 10LL, v313, v314);
L1:;
                /* phi status <- (bb14: v49) (bb78: v310) */
                /* phi v318 <- (bb14: v50) (bb78: v311) */
                /* phi v319 <- (bb14: v78) (bb78: v316) */
                /* phi v320 <- (bb14: v76) (bb78: v312) */
                /* phi v321 <- (bb14: v72) (bb78: v313) */
                /* phi v322 <- (bb14: v55) (bb78: v314) */
                v324 = ((long long (*)(long long, long long, long long, long long, long long, long long))exit)(status, v318, v320, v319, v321, v322);
                goto L0;
            } else {
                v131 = ((long long (*)(long long, long long, long long, long long, long long, long long))_pei386_runtime_relocator)(v25, v127, 0LL, 5368713232LL, 4LL, 0LL);
                v133 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v25, v127, 0LL, 5368717184LL, 4LL, 0LL);
                v134 = (*((int64_t *)(5368726688LL)));
                *((int64_t *)(v134)) = v133;
                v137 = ((long long (*)(long long, long long, long long, long long, long long, long long))_set_invalid_parameter_handler)(v25, v127, v134, 5368713216LL, 4LL, 0LL);
                v138 = ((long long (*)(long long, long long, long long, long long, long long, long long))_fpreset)(v25, v127, v134, 5368713216LL, 4LL, 0LL);
                v139 = (*((int64_t *)(5368726640LL)));
                *((int32_t *)(v139)) = 1LL;
                v141 = (*((int64_t *)(5368726656LL)));
                *((int32_t *)(v141)) = 1LL;
                v143 = (*((int64_t *)(5368726672LL)));
                *((int32_t *)(v143)) = 1LL;
                v145 = ((S_140001020_v145_t *)((*((int64_t *)(5368726528LL)))));
                v147 = (*((int16_t *)(((int64_t)(v145)))));
                v148 = (v147 != 23117LL);
                if (v148) {
                } else {
                    v149 = (((int64_t)(v145)) + 60LL);
                    v150 = v145->field_3c;
                    v152 = ((S_140001020_v152_t *)((((int64_t)(v145)) + v150)));
                    v153 = (*((int32_t *)(((int64_t)(v152)))));
                    v154 = (v153 != 17744LL);
                    if (v154) {
                    } else {
                        v155 = ((void *)((((int64_t)(v152)) + 24LL)));
                        v156 = v152->field_18;
                        v158 = (v156 == 267LL);
                        if (v158) {
                            v167 = ((void *)((((int64_t)(v152)) + 116LL)));
                            v168 = v152->field_74;
                            v169 = (v168 <= 14LL);
                            if (v169) {
                            } else {
                                v170 = ((void *)((((int64_t)(v152)) + 232LL)));
                                v171 = v152->field_e8;
                                (/* opaque: setne */ 0);
                            }
                        } else {
                            v159 = (v156 != 523LL);
                            if (v159) {
                            } else {
                                v160 = ((void *)((((int64_t)(v152)) + 132LL)));
                                v161 = v152->field_84;
                                v162 = (v161 <= 14LL);
                                if (v162) {
                                } else {
                                    v163 = ((void *)((((int64_t)(v152)) + 248LL)));
                                    v164 = v152->field_f8;
                                    (/* opaque: setne */ 0);
                                }
                            }
                        }
                    }
                }
                /* phi v174 <- (bb33: v127) (bb34: v127) (bb36: v127) (bb37: v127) (bb38: 0) (bb75: v127) (bb76: 0) */
                /* phi v175 <- (bb33: v134) (bb34: v150) (bb36: v156) (bb37: v156) (bb38: v156) (bb75: v156) (bb76: v156) */
                /* phi v176 <- (bb33: 0) (bb34: 0) (bb36: 0) (bb37: 0) (bb38: v164) (bb75: 0) (bb76: 0) */
                v177 = (*((int64_t *)(5368726624LL)));
                *((int32_t *)(5368741896LL)) = v174;
                v179 = (*((int32_t *)(v177)));
                v182 = (v179 != 0LL);
                if (v182) {
                    /* phi v302 <- (bb39: v25) (bb63: v295) */
                    /* phi v303 <- (bb39: v174) (bb63: v296) */
                    /* phi v304 <- (bb39: v175) (bb63: v297) */
                    /* phi v305 <- (bb39: v179) (bb63: v298) */
                    /* phi v306 <- (bb39: v176) (bb63: v299) */
                    v308 = ((long long (*)(long long, long long, long long, long long, long long, long long))__set_app_type)(v302, v303, v304, 2LL, v305, v306);
                } else {
                    v184 = ((long long (*)(long long, long long, long long, long long, long long, long long))__set_app_type)(v25, v174, v175, 1LL, v179, v176);
                }
                /* phi n <- (bb40: v25) (bb65: v302) */
                /* phi v186 <- (bb40: v174) (bb65: v303) */
                /* phi v187 <- (bb40: 1) (bb65: 2) */
                /* phi v188 <- (bb40: v175) (bb65: v304) */
                /* phi v189 <- (bb40: v179) (bb65: v305) */
                /* phi v190 <- (bb40: v176) (bb65: v306) */
                v191 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__p__fmode)(n, v186, v188, v187, v189, v190)));
                v192 = (*((int64_t *)(5368726832LL)));
                v194 = (*((int32_t *)(v192)));
                *((int32_t *)(((int64_t)(v191)))) = v194;
                v196 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__p__commode)(n, v186, v194, v187, v189, v190)));
                v197 = (*((int64_t *)(5368726800LL)));
                v199 = (*((int32_t *)(v197)));
                *((int32_t *)(((int64_t)(v196)))) = v199;
                v201 = ((long long (*)(long long, long long, long long, long long, long long, long long))_setargv)(n, v186, v199, v187, v189, v190);
                /* dac: structuring fallback */
            }
        } else {
            *((int32_t *)(5368741892LL)) = 1LL;
            /* phi v49 <- (bb8: v25) (bb70: v281) */
            /* phi v50 <- (bb8: v21) (bb70: v282) */
            /* phi v52 <- (bb8: v29) (bb70: v287) */
            /* phi v53 <- (bb8: v33) (bb70: v285) */
            /* phi v54 <- (bb8: v34) (bb70: v284) */
            /* phi v55 <- (bb8: v35) (bb70: v227) */
            v57 = (v39 == 0LL);
            if (v57) {
                (/* opaque: xchg */ 0);
            }
            v58 = (*((int64_t *)(5368726576LL)));
            v60 = (*((int64_t *)(v58)));
            v63 = (v60 == 0LL);
            if (v63) {
            } else {
                v67 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v49, v50, 2LL, 0LL, 0LL, v55);
            }
            /* phi v68 <- (bb10: v52) (bb11: 0) */
            /* phi v69 <- (bb10: v53) (bb11: 2) */
            /* phi v70 <- (bb10: v54) (bb11: 0) */
            v71 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__p___initenv)(v49, v50, v69, v68, v70, v55)));
            v72 = (*((int64_t *)(5368741904LL)));
            v74 = (*((int32_t *)(5368741920LL)));
            *((int64_t *)(((int64_t)(v71)))) = v72;
            v76 = (*((int64_t *)(5368741912LL)));
            v78 = ((long long (*)(long long, long long, long long, long long, long long, long long))main)(v49, v50, v76, v74, v72, v55);
            v79 = (*((int32_t *)(5368741896LL)));
            v82 = (v79 == 0LL);
            if (v82) {
                goto L1;
            } else {
                v83 = (*((int32_t *)(5368741892LL)));
                v86 = (v83 == 0LL);
                if (v86) {
                    v113 = ((void *)((((int64_t)(v17)) + 60LL)));
                    v17->field_3c = v78;
                    v114 = ((long long (*)(long long, long long, long long, long long, long long, long long))_cexit)(v49, v50, v83, v79, v72, v55);
                    v115 = ((void *)((((int64_t)(v17)) + 60LL)));
                    v116 = v17->field_3c;
                }
                /* phi v87 <- (bb15: v78) (bb21: v116) */
                v88 = ((void *)((((int64_t)(v17)) + 88LL)));
                v89 = v17->field_58;
                v90 = ((void *)((((int64_t)(v88)) + 8LL)));
                v92 = (*((int64_t *)(((int64_t)(v90)))));
                v93 = ((void *)((((int64_t)(v90)) + 8LL)));
                v95 = (*((int64_t *)(((int64_t)(v93)))));
                v96 = ((void *)((((int64_t)(v93)) + 8LL)));
                v98 = (*((int64_t *)(((int64_t)(v96)))));
                v99 = ((void *)((((int64_t)(v96)) + 8LL)));
                v101 = (*((int64_t *)(((int64_t)(v99)))));
                v102 = ((void *)((((int64_t)(v99)) + 8LL)));
                v104 = (*((int64_t *)(((int64_t)(v102)))));
                v105 = ((void *)((((int64_t)(v102)) + 8LL)));
                v107 = (*((int64_t *)(((int64_t)(v105)))));
                v108 = ((void *)((((int64_t)(v105)) + 8LL)));
                v110 = (*((int64_t *)(((int64_t)(v108)))));
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
/* convention: ms-x64 (score 0.85) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t WinMainCRTStartup(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    int64_t v2 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = arg1;
    int64_t v7 = arg0;
    int64_t v8 = arg2;
    int64_t v9 = arg3;
    int64_t v10 = 0LL;

    v2 = (*((int64_t *)(5368726624LL)));
    *((int32_t *)(v2)) = 1LL;
    v10 = ((long long (*)(long long, long long, long long, long long, long long, long long))__tmainCRTStartup)(v4, v5, v6, v7, v8, v9);
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
/* convention: ms-x64 (score 0.85) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t mainCRTStartup(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    int64_t v2 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = arg1;
    int64_t v7 = arg0;
    int64_t v8 = arg2;
    int64_t v9 = arg3;
    int64_t v10 = 0LL;

    v2 = (*((int64_t *)(5368726624LL)));
    *((int32_t *)(v2)) = 0LL;
    v10 = ((long long (*)(long long, long long, long long, long long, long long, long long))__tmainCRTStartup)(v4, v5, v6, v7, v8, v9);
    return v10;
}

/* dac-recovered forwarding thunk */
/* address: 0x140001460 */
/* end: 0x140001490 */
/* confidence: 1.00 (Observed) */
/* tail-call: _crt_atexit (0x1400029c8) */
void atexit(void) {
    _crt_atexit();
}

/* dac-recovered function */
/* address: 0x140001490 */
/* end: 0x140001540 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 9 */
/* goto_count: 1 */
/* label_count: 1 */
/* irreducible: false */
/* convention: ms-x64 (score 0.10) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 4 */
/* struct_layouts: pointer=0 stack=1 */
/* switch_tables: 0 */
void __gcc_register_frame(void) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    void * v6 = ((void *)(0LL));
    int64_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int8_t v17 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    void * v24 = ((void *)(0LL));
    int64_t v25 = 0LL;
    void * v28 = ((void *)(0LL));
    int64_t v29 = 0LL;
    void * v30 = ((void *)(0LL));
    int64_t v31 = 0LL;
    int8_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v39 = 0LL;
    void * v41 = ((void *)(0LL));
    int64_t v42 = 0LL;
    void * v43 = ((void *)(0LL));
    int64_t v45 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 56LL)));
    v6 = ((void *)((((int64_t)(v5)) + 48LL)));
    v14 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v9, v10, v11, 5368725504LL, v12, v13);
    v17 = (v14 == 0LL);
    if (v17) {
        *((int64_t *)(5368721408LL)) = 5368714368LL;
L0:;
        /* phi v35 <- (bb5: v31) (bb8: 5368714352) */
        /* phi v36 <- (bb5: v20) (bb8: v13) */
        v39 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v9, v10, 5368741984LL, 5368729600LL, v35, v36);
    } else {
        v19 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v9, v10, v11, 5368725504LL, v12, v13);
        v20 = (*((int64_t *)(5368746672LL)));
        *((int64_t *)(5368741952LL)) = v19;
        v24 = ((void *)((((int64_t)(v6)) + -16LL)));
        *((int64_t *)(((int64_t)(v24)))) = v20;
        v25 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v9, v10, 5368725523LL, v14, v12, v20);
        v28 = ((void *)((((int64_t)(v6)) + -8LL)));
        *((int64_t *)(((int64_t)(v28)))) = v25;
        v29 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v9, v10, 5368725545LL, v14, v12, v20);
        v30 = ((void *)((((int64_t)(v6)) + -8LL)));
        v31 = (*((int64_t *)(((int64_t)(v30)))));
        *((int64_t *)(5368721408LL)) = v29;
        v34 = (v31 == 0LL);
        if (v34) {
        } else {
            goto L0;
        }
    }
    v41 = ((void *)((((int64_t)(v5)) + 56LL)));
    v42 = (*((int64_t *)(((int64_t)(v41)))));
    v43 = ((void *)((((int64_t)(v41)) + 8LL)));
    v45 = (*((int64_t *)(((int64_t)(v43)))));
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140001540 */
/* end: 0x140001580 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 5 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.25) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 1 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t __gcc_deregister_frame(void) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v4 = ((void *)(0LL));
    int64_t v5 = 0LL;
    int8_t v8 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int8_t v20 = 0LL;
    void * v21 = ((void *)(0LL));
    int64_t v22 = 0LL;
    void * v25 = ((void *)(0LL));
    int64_t v26 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v4 = ((void *)((((int64_t)(v1)) - 32LL)));
    v5 = (*((int64_t *)(5368721408LL)));
    v8 = (v5 == 0LL);
    if (v8) {
    } else {
        v15 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v10, v11, v12, 5368729600LL, v13, v14);
    }
    /* phi v16 <- (bb0: v5) (bb1: v15) */
    v17 = (*((int64_t *)(5368741952LL)));
    v20 = (v17 == 0LL);
    if (v20) {
        v25 = ((void *)((((int64_t)(v4)) + 32LL)));
        v26 = (*((int64_t *)(((int64_t)(v25)))));
        return v16;
    } else {
        v21 = ((void *)((((int64_t)(v4)) + 32LL)));
        v22 = (*((int64_t *)(((int64_t)(v21)))));
        /* dac: structuring fallback */
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
/* convention: ms-x64 (score 0.85) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t __do_global_dtors(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    int64_t v2 = 0LL;
    int64_t v4 = 0LL;
    int8_t v7 = 0LL;
    int64_t v8 = arg1;
    int64_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = arg0;
    int64_t v13 = arg2;
    int64_t v14 = arg3;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v18 = 0LL;
    int64_t v21 = 0LL;
    int8_t v24 = 0LL;
    int64_t v25 = 0LL;

    v2 = (*((int64_t *)(5368721424LL)));
    v4 = (*((int64_t *)(v2)));
    v7 = (v4 == 0LL);
    if (v7) {
    } else {
        while (1) {
            /* phi v9 <- (bb1: v8) (bb3: v18) */
            v15 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v10, v11, v9, v12, v13, v14);
            v16 = (*((int64_t *)(5368721424LL)));
            v18 = (v16 + 8LL);
            v21 = (*((int64_t *)(v18)));
            *((int64_t *)(5368721424LL)) = v18;
            v24 = (v21 != 0LL);
            if (v24) {
                continue;
            }
        }
    }
    /* phi v25 <- (bb0: v4) (bb3: v21) */
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
/* convention: ms-x64 (score 0.20) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 2 */
/* struct_layouts: pointer=0 stack=1 */
/* switch_tables: 0 */
void __do_global_ctors(void) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    int64_t v6 = 0LL;
    int64_t v8 = 0LL;
    int8_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int8_t v16 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int8_t v32 = 0LL;
    void * v34 = ((void *)(0LL));
    int64_t v35 = 0LL;
    void * v36 = ((void *)(0LL));
    int64_t v38 = 0LL;
    int64_t i = 0LL;
    int64_t v43 = 0LL;
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
    v8 = (*((int64_t *)(v6)));
    v11 = (v8 == -1LL);
    if (v11) {
        while (1) {
            /* phi i <- (bb7: 0) (bb8: v43) */
            v43 = (i + 1LL);
            v47 = (v43 * 8LL);
            v48 = (v6 + v47);
            v49 = (*((int64_t *)(v48)));
            v50 = (v49 != 0LL);
            if (v50) {
                continue;
            } else {
                break;
            }
        }
    }
    /* phi v13 <- (bb0: v8) (bb9: i) */
    /* phi v14 <- (bb0: v12) (bb9: v43) */
    v16 = (v13 == 0LL);
    if (v16) {
    } else {
        v18 = (v13 - 1LL);
        v19 = (v13 * 8LL);
        v20 = (v6 + v19);
        v22 = (v13 - v18);
        v23 = (v6 + -8LL);
        v24 = (v22 * 8LL);
        v25 = (v23 + v24);
        while (1) {
            /* phi v27 <- (bb2: v20) (bb4: v31) */
            v30 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v28, v25, v6, v18, v14, v29);
            v31 = (v27 - 8LL);
            v32 = (v31 != v25);
            if (v32) {
                continue;
            }
        }
    }
    v34 = ((void *)((((int64_t)(v5)) + 40LL)));
    v35 = (*((int64_t *)(((int64_t)(v34)))));
    v36 = ((void *)((((int64_t)(v34)) + 8LL)));
    v38 = (*((int64_t *)(((int64_t)(v36)))));
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140001650 */
/* end: 0x140001670 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 3 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.55) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int32_t __main(void) {
    int32_t v0 = 0LL;
    int8_t v3 = 0LL;

    v0 = (*((int32_t *)(5368742048LL)));
    v3 = (v0 == 0LL);
    if (v3) {
        *((int32_t *)(5368742048LL)) = 1LL;
        /* dac: structuring fallback */
    } else {
        return v0;
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
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t _setargv(void) {
    int64_t v0 = 0LL;

    return 0LL;
}

/* dac-recovered function */
/* address: 0x140001680 */
/* end: 0x1400016a0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 4 */
/* goto_count: 1 */
/* label_count: 1 */
/* irreducible: false */
/* convention: ms-x64 (score 0.30) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t __dyn_tls_dtor(void) {
    int64_t v0 = 0LL;
    int8_t v1 = 0LL;
    int8_t v3 = 0LL;
    int64_t v4 = 0LL;

    v1 = (v0 == 3LL);
    if (v1) {
L0:;
        /* dac: structuring fallback */
    } else {
        v3 = (v0 == 0LL);
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
/* convention: ms-x64 (score 0.85) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 2 */
/* struct_layouts: pointer=0 stack=1 */
/* switch_tables: 0 */
int64_t __dyn_tls_init(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    int64_t v6 = 0LL;
    int32_t v8 = 0LL;
    int8_t v9 = 0LL;
    int64_t v10 = arg1;
    int8_t v11 = 0LL;
    int8_t v12 = 0LL;
    void * v13 = ((void *)(0LL));
    int64_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int64_t v17 = 0LL;
    void * v20 = ((void *)(0LL));
    int64_t v21 = 0LL;
    void * v22 = ((void *)(0LL));
    int64_t v24 = 0LL;
    int8_t v29 = 0LL;
    void * v30 = ((void *)(0LL));
    int64_t v31 = 0LL;
    int8_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = arg0;
    int64_t v37 = arg2;
    int64_t v38 = arg3;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    void * v41 = ((void *)(0LL));
    int8_t v42 = 0LL;
    void * v43 = ((void *)(0LL));
    int64_t v44 = 0LL;
    void * v45 = ((void *)(0LL));
    int64_t v47 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 40LL)));
    v6 = (*((int64_t *)(5368726480LL)));
    v8 = (*((int32_t *)(v6)));
    v9 = (v8 == 2LL);
    if (v9) {
    } else {
        *((int32_t *)(v6)) = 2LL;
    }
    v11 = (v10 == 2LL);
    if (v11) {
        v29 = (5368726976LL == 5368726976LL);
        if (v29) {
L0:;
            v20 = ((void *)((((int64_t)(v5)) + 40LL)));
            v21 = (*((int64_t *)(((int64_t)(v20)))));
            v22 = ((void *)((((int64_t)(v20)) + 8LL)));
            v24 = (*((int64_t *)(((int64_t)(v22)))));
            return v6;
        } else {
            while (1) {
                /* phi v30 <- (bb7: 5368726976) (bb10: v41) */
                v31 = (*((int64_t *)(((int64_t)(v30)))));
                v34 = (v31 == 0LL);
                if (v34) {
                } else {
                    v39 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v35, 5368726976LL, v10, v36, v37, v38);
                }
                /* phi v40 <- (bb8: v31) (bb9: v39) */
                v41 = ((void *)((((int64_t)(v30)) + 8LL)));
                v42 = (((int64_t)(v41)) != 5368726976LL);
                if (v42) {
                    continue;
                } else {
                    break;
                }
            }
            v43 = ((void *)((((int64_t)(v5)) + 40LL)));
            v44 = (*((int64_t *)(((int64_t)(v43)))));
            v45 = ((void *)((((int64_t)(v43)) + 8LL)));
            v47 = (*((int64_t *)(((int64_t)(v45)))));
            return v40;
        }
    } else {
        v12 = (v10 == 1LL);
        if (v12) {
            v13 = ((void *)((((int64_t)(v5)) + 40LL)));
            v14 = (*((int64_t *)(((int64_t)(v13)))));
            v15 = ((void *)((((int64_t)(v13)) + 8LL)));
            v17 = (*((int64_t *)(((int64_t)(v15)))));
            /* dac: structuring fallback */
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
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t __tlregdtor(void) {
    int64_t v0 = 0LL;

    return 0LL;
}

/* dac-recovered function */
/* address: 0x140001730 */
/* end: 0x140001830 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 6 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.85) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 2 */
/* struct_layouts: pointer=1 stack=1 */
/* switch_tables: 1 */
int64_t _matherr(S_140001730_v6_t * arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    S_140001730_v6_t * v6 = arg0;
    int32_t v7 = 0LL;
    int8_t v8 = 0LL;
    int32_t v9 = 0LL;
    int32_t v12 = 0LL;
    int64_t v13 = 0LL;
    int32_t v14 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = arg1;
    int64_t v24 = arg2;
    int64_t v25 = arg3;
    int64_t v26 = 0LL;
    int64_t v31 = 0LL;
    void * v33 = ((void *)(0LL));
    int64_t v34 = 0LL;
    void * v35 = ((void *)(0LL));
    int64_t v37 = 0LL;

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
        (/* opaque: movsd */ 0);
        (/* opaque: movsd */ 0);
        (/* opaque: movsd */ 0);
        v18 = (((int64_t)(v6)) + 8LL);
        v19 = v6->field_8;
        v26 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v22, v19, v23, 2LL, v24, v25);
        (/* opaque: movsd */ 0);
        (/* opaque: movsd */ 0);
        (/* opaque: movsd */ 0);
        v31 = ((long long (*)(long long, long long, long long, long long, long long, long long))fprintf)(v22, v19, 5368725944LL, v26, 5368725926LL, v19);
        (/* opaque: movaps */ 0);
        (/* opaque: movaps */ 0);
        (/* opaque: movaps */ 0);
        v33 = ((void *)((((int64_t)(v5)) + 120LL)));
        v34 = (*((int64_t *)(((int64_t)(v33)))));
        v35 = ((void *)((((int64_t)(v33)) + 8LL)));
        v37 = (*((int64_t *)(((int64_t)(v35)))));
        return 0LL;
    } else {
        v9 = (*((int32_t *)(((int64_t)(v6)))));
        v12 = (v9 * 4LL);
        v13 = (5368725988LL + v12);
        v14 = (*((int32_t *)(v13)));
        /* recovered switch table at block 1 (arm resolution pending) */
        switch (v9) {
            default: {
                /* dac: structuring fallback */
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
    void * v8 = ((void *)(0LL));
    void * v11 = ((void *)(0LL));
    int64_t v12 = arg2;
    void * v13 = ((void *)(0LL));
    int64_t v14 = arg3;
    int64_t v16 = arg1;
    void * v17 = ((void *)(0LL));
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v22 = 0LL;
    void * v23 = ((void *)(0LL));
    int64_t v24 = 0LL;
    int64_t v27 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((S_140001830_v5_t *)((((int64_t)(v3)) - 56LL)));
    v8 = ((void *)((((int64_t)(v5)) + 88LL)));
    v11 = ((void *)((((int64_t)(v5)) + 96LL)));
    v5->field_60 = v12;
    v13 = ((void *)((((int64_t)(v5)) + 104LL)));
    v5->field_68 = v14;
    v5->field_58 = v16;
    v17 = ((void *)((((int64_t)(v5)) + 40LL)));
    v5->field_28 = ((int64_t)(v8));
    v19 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v18, v2, v16, 2LL, v12, v14);
    v22 = ((long long (*)(long long, long long, long long, long long, long long, long long))fprintf)(v18, v2, 5368726016LL, v19, v12, v14);
    v23 = ((void *)((((int64_t)(v5)) + 40LL)));
    v24 = v5->field_28;
    v27 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v18, v24, 5368726016LL, 2LL, v12, v14);
    v31 = ((long long (*)(long long, long long, long long, long long, long long, long long))vfprintf)(v18, v24, v6, v27, v24, v14);
    v32 = ((long long (*)(long long, long long, long long, long long, long long, long long))abort)(v18, v24, v6, v27, v24, v14);
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140001890 */
/* end: 0x140001a00 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 16 */
/* goto_count: 3 */
/* label_count: 3 */
/* irreducible: true */
/* convention: ms-x64 (score 0.85) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 6 */
/* struct_layouts: pointer=5 stack=1 */
/* switch_tables: 0 */
int64_t mark_section_writable(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    int64_t v6 = 0LL;
    S_140001890_v7_t * v7 = ((S_140001890_v7_t *)(0LL));
    int32_t v8 = 0LL;
    int64_t v10 = arg0;
    int8_t v13 = 0LL;
    int64_t v14 = arg3;
    int64_t v15 = arg2;
    int64_t v16 = arg1;
    int64_t v17 = 0LL;
    int64_t v20 = 0LL;
    S_140001890_v21_t * v21 = ((S_140001890_v21_t *)(0LL));
    int64_t i = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    int8_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v30 = 0LL;
    int32_t v31 = 0LL;
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
    S_140001890_v47_t * v47 = ((S_140001890_v47_t *)(0LL));
    int8_t v50 = 0LL;
    int64_t v51 = 0LL;
    int32_t v53 = 0LL;
    int32_t v54 = 0LL;
    int32_t v56 = 0LL;
    S_140001890_v57_t * v57 = ((S_140001890_v57_t *)(0LL));
    void * v58 = ((void *)(0LL));
    int64_t v59 = 0LL;
    int64_t v60 = 0LL;
    int32_t v61 = 0LL;
    int32_t v64 = 0LL;
    int64_t v66 = 0LL;
    void * v68 = ((void *)(0LL));
    int64_t v70 = 0LL;
    void * v71 = ((void *)(0LL));
    int64_t v72 = 0LL;
    int8_t v74 = 0LL;
    void * v75 = ((void *)(0LL));
    int32_t v76 = 0LL;
    void * v84 = ((void *)(0LL));
    int64_t v85 = 0LL;
    void * v87 = ((void *)(0LL));
    int64_t v88 = 0LL;
    int64_t v92 = 0LL;
    S_140001890_v93_t * v93 = ((S_140001890_v93_t *)(0LL));
    void * v94 = ((void *)(0LL));
    void * v96 = ((void *)(0LL));
    int64_t v97 = 0LL;
    int8_t v99 = 0LL;
    int64_t v100 = 0LL;
    int64_t v103 = 0LL;
    int32_t v104 = 0LL;
    int32_t v105 = 0LL;
    int32_t v106 = 0LL;
    int64_t v107 = 0LL;
    int64_t v109 = 0LL;
    int32_t v110 = 0LL;
    int64_t v113 = 0LL;
    void * v114 = ((void *)(0LL));
    int64_t v115 = 0LL;
    int64_t v117 = 0LL;
    int64_t v118 = 0LL;
    int64_t v119 = 0LL;
    int64_t v122 = 0LL;
    int64_t v123 = 0LL;
    void * v124 = ((void *)(0LL));
    int64_t v125 = 0LL;
    void * v126 = ((void *)(0LL));
    int64_t v128 = 0LL;
    void * v129 = ((void *)(0LL));
    int64_t v131 = 0LL;
    int64_t v134 = 0LL;
    int64_t v136 = 0LL;
    int64_t v137 = 0LL;
    int64_t v138 = 0LL;
    int64_t v139 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 8LL)));
    *((int64_t *)(((int64_t)(v5)))) = v6;
    v7 = ((S_140001890_v7_t *)((((int64_t)(v5)) - 80LL)));
    v8 = (*((int32_t *)(5368742148LL)));
    v13 = (v8 <= 0LL);
    if (v13) {
        /* phi v134 <- (bb0: v2) (bb19: v47) */
        /* phi v136 <- (bb0: v10) (bb19: v93) */
        /* phi v137 <- (bb0: v14) (bb19: v93) */
        /* phi v138 <- (bb0: v15) (bb19: 64) */
        /* phi v139 <- (bb0: v16) (bb19: v100) */
L2:;
        /* phi v40 <- (bb4: v2) (bb20: v134) */
        /* phi v41 <- (bb4: v8) (bb20: 0) */
        /* phi v42 <- (bb4: v10) (bb20: v136) */
        /* phi v43 <- (bb4: v37) (bb20: v137) */
        /* phi v44 <- (bb4: v35) (bb20: v138) */
        /* phi v45 <- (bb4: v36) (bb20: v139) */
        v47 = ((S_140001890_v47_t *)(((long long (*)(long long, long long, long long, long long, long long, long long))__mingw_GetSectionForAddress)(v40, v41, v45, v42, v44, v43)));
        v50 = (((int64_t)(v47)) == 0LL);
        if (v50) {
L0:;
            /* phi v118 <- (bb6: v42) (bb21: v56) */
            /* phi v119 <- (bb6: v44) (bb21: v115) */
            v122 = ((long long (*)(long long, long long, long long, long long, long long, long long))__report_error)(((int64_t)(v47)), v41, v118, 5368726048LL, v119, v43);
            /* dac: structuring fallback */
        } else {
            v51 = (*((int64_t *)(5368742152LL)));
            v53 = (v41 * 4LL);
            v54 = (v41 + v53);
            v56 = (v54 << 3LL);
            v57 = ((S_140001890_v57_t *)((v51 + v56)));
            v58 = ((void *)((((int64_t)(v57)) + 32LL)));
            v57->field_20 = ((int64_t)(v47));
            *((int32_t *)(((int64_t)(v57)))) = 0LL;
            v59 = ((long long (*)(long long, long long, long long, long long, long long, long long))_GetPEImageBase)(((int64_t)(v47)), v41, v45, v42, v44, v43);
            v60 = (((int64_t)(v47)) + 12LL);
            v61 = v47->field_c;
            v64 = (v59 + v61);
            v66 = (*((int64_t *)(5368742152LL)));
            v68 = ((void *)((((int64_t)(v7)) + 32LL)));
            v70 = (v66 + 24LL);
            v71 = ((void *)((v70 + v56)));
            *((int64_t *)(((int64_t)(v71)))) = v64;
            v72 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(((int64_t)(v47)), v41, ((int64_t)(v68)), v64, 48LL, v43);
            v74 = (v72 == 0LL);
            if (v74) {
                v107 = (*((int64_t *)(5368742152LL)));
                v109 = (((int64_t)(v47)) + 8LL);
                v110 = v47->field_8;
                v113 = (v107 + 24LL);
                v114 = ((void *)((v113 + v56)));
                v115 = (*((int64_t *)(((int64_t)(v114)))));
                v117 = ((long long (*)(long long, long long, long long, long long, long long, long long))__report_error)(((int64_t)(v47)), v41, v110, 5368726080LL, v115, v43);
                goto L0;
            } else {
                v75 = ((void *)((((int64_t)(v7)) + 68LL)));
                v76 = v7->field_44;
                /* dac: structuring fallback */
            }
        }
    } else {
        v17 = (*((int64_t *)(5368742152LL)));
        v20 = (v17 + 24LL);
        while (1) {
            /* phi v21 <- (bb1: v20) (bb4: v38) */
            /* phi i <- (bb1: 0) (bb4: v37) */
            /* phi v23 <- (bb1: v16) (bb4: v36) */
            v24 = (*((int64_t *)(((int64_t)(v21)))));
            v26 = (v10 < v24);
            if (v26) {
L1:;
                /* phi v35 <- (bb2: v24) (bb3: v33) */
                /* phi v36 <- (bb2: v23) (bb3: v31) */
                v37 = (i + 1LL);
                v38 = (((int64_t)(v21)) + 40LL);
                v39 = (v37 != v8);
                if (v39) {
                    continue;
                } else {
                    break;
                }
            } else {
                v27 = (((int64_t)(v21)) + 8LL);
                v28 = v21->field_8;
                v30 = (v28 + 8LL);
                v31 = (*((int32_t *)(v30)));
                v33 = (v24 + v31);
                v34 = (v10 < v33);
                if (v34) {
                    /* phi v123 <- (bb3: v21) (bb12: v104) */
                    v124 = ((void *)((((int64_t)(v7)) + 80LL)));
                    v125 = v7->field_50;
                    v126 = ((void *)((((int64_t)(v124)) + 8LL)));
                    v128 = (*((int64_t *)(((int64_t)(v126)))));
                    v129 = ((void *)((((int64_t)(v126)) + 8LL)));
                    v131 = (*((int64_t *)(((int64_t)(v129)))));
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
/* convention: ms-x64 (score 0.70) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 9 */
/* struct_layouts: pointer=5 stack=1 */
/* switch_tables: 0 */
int64_t _pei386_runtime_relocator(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
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
    int64_t v12 = 0LL;
    void * v13 = ((void *)(0LL));
    int64_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int64_t v16 = 0LL;
    void * v17 = ((void *)(0LL));
    void * v18 = ((void *)(0LL));
    int32_t src = 0LL;
    int8_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    void * v26 = ((void *)(0LL));
    int64_t v28 = 0LL;
    void * v29 = ((void *)(0LL));
    int64_t v31 = 0LL;
    void * v32 = ((void *)(0LL));
    int64_t v34 = 0LL;
    void * v35 = ((void *)(0LL));
    int64_t v37 = 0LL;
    void * v38 = ((void *)(0LL));
    int64_t v40 = 0LL;
    void * v41 = ((void *)(0LL));
    int64_t v43 = 0LL;
    void * v44 = ((void *)(0LL));
    int64_t v46 = 0LL;
    void * v47 = ((void *)(0LL));
    int64_t v49 = 0LL;
    int64_t v52 = arg1;
    int64_t v53 = arg0;
    int64_t v54 = arg2;
    int64_t v55 = arg3;
    int64_t v56 = 0LL;
    int64_t v64 = 0LL;
    int64_t dst = 0LL;
    S_140001a00_v67_t * v67 = ((S_140001a00_v67_t *)(0LL));
    void * v69 = ((void *)(0LL));
    void * v70 = ((void *)(0LL));
    int64_t v73 = 0LL;
    int8_t v74 = 0LL;
    int8_t v75 = 0LL;
    S_140001a00_v76_t * v76 = ((S_140001a00_v76_t *)(0LL));
    int64_t v77 = 0LL;
    int64_t v78 = 0LL;
    int32_t v79 = 0LL;
    int8_t v82 = 0LL;
    int64_t v83 = 0LL;
    int32_t v84 = 0LL;
    int8_t v87 = 0LL;
    int64_t v88 = 0LL;
    int32_t v89 = 0LL;
    int64_t v90 = 0LL;
    int64_t v91 = 0LL;
    int64_t v92 = 0LL;
    int32_t v93 = 0LL;
    int8_t v95 = 0LL;
    int64_t v96 = 0LL;
    int64_t v97 = 0LL;
    void * n = ((void *)(0LL));
    int8_t v101 = 0LL;
    S_140001a00_v102_t * v102 = ((S_140001a00_v102_t *)(0LL));
    int32_t v103 = 0LL;
    int64_t v105 = 0LL;
    int32_t v106 = 0LL;
    int64_t v108 = 0LL;
    int32_t v109 = 0LL;
    void * v111 = ((void *)(0LL));
    int64_t v113 = 0LL;
    void * v115 = ((void *)(0LL));
    int8_t v116 = 0LL;
    void * v117 = ((void *)(0LL));
    int64_t v118 = 0LL;
    int8_t v119 = 0LL;
    int8_t v120 = 0LL;
    int8_t v121 = 0LL;
    int16_t v122 = 0LL;
    int16_t v124 = 0LL;
    int16_t v125 = 0LL;
    void * v126 = ((void *)(0LL));
    int64_t v127 = 0LL;
    void * v129 = ((void *)(0LL));
    int8_t v130 = 0LL;
    int8_t v131 = 0LL;
    int64_t v134 = 0LL;
    void * v138 = ((void *)(0LL));
    int8_t v139 = 0LL;
    int8_t v141 = 0LL;
    int8_t v142 = 0LL;
    void * v143 = ((void *)(0LL));
    int64_t v144 = 0LL;
    void * v146 = ((void *)(0LL));
    int8_t v147 = 0LL;
    int8_t v148 = 0LL;
    int64_t v151 = 0LL;
    void * v155 = ((void *)(0LL));
    int8_t v156 = 0LL;
    int64_t v157 = 0LL;
    int64_t v159 = 0LL;
    int64_t v160 = 0LL;
    void * v162 = ((void *)(0LL));
    int64_t v164 = 0LL;
    int64_t v166 = 0LL;
    void * v170 = ((void *)(0LL));
    int8_t v171 = 0LL;
    void * v173 = ((void *)(0LL));
    int64_t v174 = 0LL;
    int32_t v175 = 0LL;
    int32_t v178 = 0LL;
    int32_t v179 = 0LL;
    void * v180 = ((void *)(0LL));
    int64_t v181 = 0LL;
    void * v183 = ((void *)(0LL));
    int8_t v185 = 0LL;
    int8_t v186 = 0LL;
    int64_t v189 = 0LL;
    void * v193 = ((void *)(0LL));
    int64_t v194 = 0LL;
    void * v195 = ((void *)(0LL));
    int64_t v198 = 0LL;
    int32_t v199 = 0LL;
    int64_t v200 = 0LL;
    int64_t v201 = 0LL;
    int64_t v203 = 0LL;
    void * v204 = ((void *)(0LL));
    int32_t v206 = 0LL;
    int8_t v209 = 0LL;
    int64_t v210 = 0LL;
    int64_t i = 0LL;
    int64_t v214 = 0LL;
    int64_t v215 = 0LL;
    S_140001a00_v217_t * v217 = ((S_140001a00_v217_t *)(0LL));
    int32_t v218 = 0LL;
    int8_t v221 = 0LL;
    int64_t v222 = 0LL;
    int64_t v223 = 0LL;
    int64_t v225 = 0LL;
    int64_t v226 = 0LL;
    int64_t v229 = 0LL;
    int64_t v230 = 0LL;
    int64_t v231 = 0LL;
    int64_t v232 = 0LL;
    int32_t v233 = 0LL;
    int8_t v234 = 0LL;
    int32_t v235 = 0LL;
    int8_t v238 = 0LL;
    int64_t v239 = 0LL;
    int32_t v240 = 0LL;
    int8_t v243 = 0LL;
    int64_t v244 = 0LL;
    int32_t v245 = 0LL;
    int8_t v248 = 0LL;
    int64_t v249 = 0LL;
    int64_t v250 = 0LL;
    int32_t v251 = 0LL;
    int64_t v252 = 0LL;
    int64_t v253 = 0LL;
    int64_t v254 = 0LL;
    int8_t v255 = 0LL;
    int64_t v256 = 0LL;
    void * n_1 = ((void *)(0LL));
    S_140001a00_v260_t * v260 = ((S_140001a00_v260_t *)(0LL));
    int64_t v261 = 0LL;
    int64_t v262 = 0LL;
    int64_t v263 = 0LL;
    int32_t v264 = 0LL;
    int32_t v266 = 0LL;
    int64_t v268 = 0LL;
    void * v269 = ((void *)(0LL));
    int32_t v270 = 0LL;
    int32_t v271 = 0LL;
    int64_t v272 = 0LL;
    void * v274 = ((void *)(0LL));
    int64_t v275 = 0LL;
    int64_t v278 = 0LL;
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
    src = (*((int32_t *)(5368742144LL)));
    v23 = (src == 0LL);
    if (v23) {
        *((int32_t *)(5368742144LL)) = 1LL;
        v56 = ((long long (*)(long long, long long, long long, long long, long long, long long))__mingw_GetSectionCount)(v12, src, v52, v53, v54, v55);
        (/* opaque: cdqe */ 0);
        v64 = ((long long (*)(long long, long long, long long, long long, long long, long long))fn_1400027e0)(v12, src, v52, v53, v54, v55);
        dst = (*((int64_t *)(5368726544LL)));
        v67 = ((S_140001a00_v67_t *)((*((int64_t *)(5368726560LL)))));
        v69 = ((void *)((((int64_t)(v17)) - v64)));
        *((int32_t *)(5368742148LL)) = 0LL;
        v70 = ((void *)((((int64_t)(v69)) + 48LL)));
        *((int64_t *)(5368742152LL)) = ((int64_t)(v70));
        v73 = (dst - ((int64_t)(v67)));
        v74 = (v73 <= 7LL);
        if (v74) {
L0:;
            /* phi v25 <- (bb0: v24) (bb4: v73) (bb10: v89) (bb31: v206) (bb36: v230) (bb40: v251) */
            v26 = ((void *)((((int64_t)(v18)) + 8LL)));
            v28 = (*((int64_t *)(((int64_t)(v26)))));
            v29 = ((void *)((((int64_t)(v26)) + 8LL)));
            v31 = (*((int64_t *)(((int64_t)(v29)))));
            v32 = ((void *)((((int64_t)(v29)) + 8LL)));
            v34 = (*((int64_t *)(((int64_t)(v32)))));
            v35 = ((void *)((((int64_t)(v32)) + 8LL)));
            v37 = (*((int64_t *)(((int64_t)(v35)))));
            v38 = ((void *)((((int64_t)(v35)) + 8LL)));
            v40 = (*((int64_t *)(((int64_t)(v38)))));
            v41 = ((void *)((((int64_t)(v38)) + 8LL)));
            v43 = (*((int64_t *)(((int64_t)(v41)))));
            v44 = ((void *)((((int64_t)(v41)) + 8LL)));
            v46 = (*((int64_t *)(((int64_t)(v44)))));
            v47 = ((void *)((((int64_t)(v44)) + 8LL)));
            v49 = (*((int64_t *)(((int64_t)(v47)))));
            return v25;
        } else {
            v75 = (v73 > 11LL);
            if (v75) {
                v235 = (*((int32_t *)(((int64_t)(v67)))));
                v238 = (v235 != 0LL);
                if (v238) {
L2:;
                    /* phi v250 <- (bb6: v76) (bb7: v76) (bb38: v67) (bb39: v67) */
                    /* phi v251 <- (bb6: v73) (bb7: v84) (bb38: v73) (bb39: v73) */
                    /* phi v252 <- (bb6: v79) (bb7: v79) (bb38: v52) (bb39: v52) */
                    /* phi v253 <- (bb6: v77) (bb7: v77) (bb38: v54) (bb39: v240) */
                    /* phi v254 <- (bb6: v78) (bb7: v78) (bb38: v235) (bb39: v235) */
                    v255 = (v250 >= dst);
                    if (v255) {
                    } else {
                        v256 = (*((int64_t *)(5368726528LL)));
                        n_1 = ((void *)((((int64_t)(v18)) + -8LL)));
                        while (1) {
                            /* phi v260 <- (bb41: v250) (bb44: v268) */
                            /* phi v261 <- (bb41: v252) (bb44: n_1) */
                            /* phi v262 <- (bb41: v253) (bb44: 4) */
                            v263 = (((int64_t)(v260)) + 4LL);
                            v264 = v260->field_4;
                            v266 = (*((int32_t *)(((int64_t)(v260)))));
                            v268 = (((int64_t)(v260)) + 8LL);
                            v269 = ((void *)((v256 + v264)));
                            v270 = (*((int32_t *)(((int64_t)(v269)))));
                            v271 = (v266 + v270);
                            v272 = (v264 + v256);
                            v274 = ((void *)((((int64_t)(v18)) + -8LL)));
                            *((int32_t *)(((int64_t)(v274)))) = v271;
                            v275 = ((long long (*)(long long, long long, long long, long long, long long, long long))mark_section_writable)(dst, src, v261, v272, v262, v254);
                            v278 = (v264 + v256);
                            v280 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))memcpy)(dst, src, ((int64_t)(n_1)), v278, 4LL, v254)));
                            v281 = (v268 < dst);
                            if (v281) {
                                continue;
                            } else {
                                break;
                            }
                        }
L1:;
                        /* phi v204 <- (bb21: v117) (bb30: n) (bb45: n_1) */
                        v206 = (*((int32_t *)(5368742148LL)));
                        v209 = (v206 <= 0LL);
                        if (v209) {
                        } else {
                            v210 = (*((int64_t *)(5368746736LL)));
                            while (1) {
                                /* phi i <- (bb32: src) (bb35: v231) */
                                /* phi v214 <- (bb32: 0) (bb35: v232) */
                                v215 = (*((int64_t *)(5368742152LL)));
                                v217 = ((S_140001a00_v217_t *)((v215 + v214)));
                                v218 = (*((int32_t *)(((int64_t)(v217)))));
                                v221 = (v218 == 0LL);
                                if (v221) {
                                } else {
                                    v222 = (((int64_t)(v217)) + 16LL);
                                    v223 = v217->field_10;
                                    v225 = (((int64_t)(v217)) + 8LL);
                                    v226 = v217->field_8;
                                    v229 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v210, i, v223, v226, v218, ((int64_t)(v204)));
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
                    v239 = (((int64_t)(v67)) + 4LL);
                    v240 = v67->field_4;
                    v243 = (v240 == 0LL);
                    if (v243) {
                        v244 = (((int64_t)(v67)) + 8LL);
                        v245 = v67->field_8;
                        v248 = (v245 != 0LL);
                        if (v248) {
L3:;
                            /* phi v88 <- (bb7: v76) (bb64: v67) */
                            /* phi v89 <- (bb7: v84) (bb64: v73) */
                            /* phi v90 <- (bb7: v77) (bb64: v240) */
                            /* phi v91 <- (bb7: v78) (bb64: v235) */
                            v92 = (v88 + 8LL);
                            v93 = (*((int32_t *)(v92)));
                            v95 = (v93 != 1LL);
                            if (v95) {
                                /* phi v199 <- (bb8: v93) (bb67: v106) */
                                /* phi v200 <- (bb8: v90) (bb67: v115) */
                                /* phi v201 <- (bb8: v91) (bb67: v113) */
                                v203 = ((long long (*)(long long, long long, long long, long long, long long, long long))__report_error)(dst, src, v199, 5368726176LL, v200, v201);
                                /* dac: structuring fallback */
                            } else {
                                v96 = (v88 + 12LL);
                                v97 = (*((int64_t *)(5368726528LL)));
                                n = ((void *)((((int64_t)(v18)) + -8LL)));
                                v101 = (v96 < dst);
                                if (v101) {
                                    while (1) {
                                        /* phi v102 <- (bb9: v96) (bb21: v118) (bb29: v164) */
                                        v103 = (*((int32_t *)(((int64_t)(v102)))));
                                        v105 = (((int64_t)(v102)) + 8LL);
                                        v106 = v102->field_8;
                                        v108 = (((int64_t)(v102)) + 4LL);
                                        v109 = v102->field_4;
                                        v111 = ((void *)((v103 + v97)));
                                        v113 = (*((int64_t *)(((int64_t)(v111)))));
                                        v115 = ((void *)((v109 + v97)));
                                        v116 = (v106 == 32LL);
                                        if (v116) {
                                            v175 = (*((int32_t *)(((int64_t)(v115)))));
                                            /* dac: structuring fallback */
                                        } else {
                                            /* dac: structuring fallback */
                                        }
                                    }
                                    goto L1;
                                } else {
                                    goto L0;
                                }
                            }
                        } else {
                            v249 = (((int64_t)(v67)) + 12LL);
L4:;
                            /* phi v76 <- (bb5: v67) (bb65: v249) */
                            /* phi v77 <- (bb5: v54) (bb65: v240) */
                            /* phi v78 <- (bb5: v55) (bb65: v235) */
                            v79 = (*((int32_t *)(((int64_t)(v76)))));
                            v82 = (v79 != 0LL);
                            if (v82) {
                                goto L2;
                            } else {
                                v83 = (((int64_t)(v76)) + 4LL);
                                v84 = v76->field_4;
                                v87 = (v84 != 0LL);
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
/* convention: ms-x64 (score 0.85) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 2 */
/* struct_layouts: pointer=1 stack=1 */
/* switch_tables: 0 */
int64_t __mingw_raise_matherr(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    S_140001d90_v1_t * v1 = ((S_140001d90_v1_t *)(0LL));
    int64_t v2 = 0LL;
    int8_t v5 = 0LL;
    void * v6 = ((void *)(0LL));
    int64_t v7 = arg0;
    void * v10 = ((void *)(0LL));
    int64_t v11 = arg1;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = arg2;
    int64_t v15 = arg3;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;

    v1 = ((S_140001d90_v1_t *)((v0 - 88LL)));
    v2 = (*((int64_t *)(5368742160LL)));
    v5 = (v2 == 0LL);
    if (v5) {
    } else {
        (/* opaque: movsd */ 0);
        (/* opaque: unpcklpd */ 0);
        v6 = ((void *)((((int64_t)(v1)) + 32LL)));
        v1->field_20 = v7;
        v10 = ((void *)((((int64_t)(v1)) + 40LL)));
        v1->field_28 = v11;
        (/* opaque: movaps */ 0);
        (/* opaque: movsd */ 0);
        v16 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v12, v13, v11, ((int64_t)(v6)), v14, v15);
    }
    /* phi v17 <- (bb0: v2) (bb2: v16) */
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
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140001de0 */
/* end: 0x140001f80 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 18 */
/* goto_count: 7 */
/* label_count: 3 */
/* irreducible: false */
/* convention: ms-x64 (score 0.85) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=1 stack=0 */
/* switch_tables: 1 */
int64_t __mingw_SEH_error_handler(S_140001de0_v2_t * arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    S_140001de0_v2_t * v2 = arg0;
    int64_t v3 = 0LL;
    int8_t v4 = 0LL;
    int8_t v5 = 0LL;
    int8_t v6 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int8_t v10 = 0LL;
    int32_t v11 = 0LL;
    int8_t v13 = 0LL;
    int8_t v14 = 0LL;
    int32_t v15 = 0LL;
    int8_t v16 = 0LL;
    int32_t v18 = 0LL;
    int64_t v19 = 0LL;
    int32_t v20 = 0LL;
    int8_t v23 = 0LL;
    int8_t v24 = 0LL;
    int8_t v25 = 0LL;
    int64_t v26 = arg1;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = arg2;
    int64_t v32 = arg3;
    int64_t v33 = 0LL;
    int8_t v34 = 0LL;
    int8_t v36 = 0LL;
    int64_t v38 = 0LL;
    int64_t v41 = 0LL;
    int64_t v45 = 0LL;
    int8_t v46 = 0LL;
    int8_t v48 = 0LL;
    int64_t v50 = 0LL;
    int64_t v53 = 0LL;
    int64_t v56 = 0LL;

    v3 = (((int64_t)(v2)) + 4LL);
    v4 = v2->field_4;
    v5 = (v4 & 2LL);
    v6 = (v5 != 0LL);
    if (v6) {
L0:;
        /* phi v56 <- (bb16: 0) (bb34: 1) (bb36: v11) */
        return v56;
    } else {
        v8 = (*((int64_t *)(((int64_t)(v2)))));
        v9 = (4848615423LL & v8);
        v10 = (v9 == 541541187LL);
        if (v10) {
            goto L0;
        } else {
            v11 = (*((int32_t *)(((int64_t)(v2)))));
            v13 = (v11 > -1073741674LL);
            if (v13) {
L1:;
                goto L1;
            } else {
                v14 = (v11 <= -1073741685LL);
                if (v14) {
                    v23 = (v11 == -1073741819LL);
                    if (v23) {
                        v45 = ((long long (*)(long long, long long, long long, long long, long long, long long))signal)(v29, v30, 0LL, 11LL, v31, v32);
                        v46 = (v45 == 1LL);
                        if (v46) {
                            v53 = ((long long (*)(long long, long long, long long, long long, long long, long long))signal)(v29, v30, 1LL, 11LL, v31, v32);
L2:;
                            goto L2;
                        } else {
                            v48 = (v45 == 0LL);
                            if (v48) {
                                goto L1;
                            } else {
                                v50 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v29, v30, 0LL, 11LL, v31, v32);
                                goto L2;
                            }
                        }
                    } else {
                        /* dac: structuring fallback */
                    }
                    goto L0;
                } else {
                    v15 = (v11 + 1073741683LL);
                    v16 = (v15 > 9LL);
                    if (v16) {
                        goto L2;
                    } else {
                        v18 = (v15 * 4LL);
                        v19 = (5368726368LL + v18);
                        v20 = (*((int32_t *)(v19)));
                        /* recovered switch table at block 5 */
                        switch (v15) {
                            case 5LL: {
                                goto L3;
                            }
                            case 9LL: {
                                goto L4;
                            }
                            default: {
                                /* dac: structuring fallback */
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
    S_140001f80_v5_t * v5 = ((S_140001f80_v5_t *)(0LL));
    int32_t v7 = 0LL;
    int32_t v11 = 0LL;
    int8_t v12 = 0LL;
    int8_t v13 = 0LL;
    int8_t v14 = 0LL;
    int32_t v15 = 0LL;
    int8_t v16 = 0LL;
    int32_t v18 = 0LL;
    int64_t v19 = 0LL;
    int32_t v20 = 0LL;
    int8_t v23 = 0LL;
    int8_t v24 = 0LL;
    int8_t v25 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int8_t v33 = 0LL;
    int8_t v35 = 0LL;
    int64_t v37 = 0LL;
    int64_t v40 = 0LL;
    int8_t v41 = 0LL;
    int64_t v44 = 0LL;
    int8_t v45 = 0LL;
    int8_t v47 = 0LL;
    int64_t v49 = 0LL;
    int64_t v52 = 0LL;
    int64_t v53 = 0LL;
    int8_t v56 = 0LL;
    void * v58 = ((void *)(0LL));
    int64_t v59 = 0LL;
    void * v63 = ((void *)(0LL));
    int64_t v64 = 0LL;
    void * v68 = ((void *)(0LL));
    int64_t v69 = 0LL;
    int64_t v72 = 0LL;
    int8_t v73 = 0LL;
    int8_t v74 = 0LL;
    int8_t v75 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 32LL)));
    v5 = ((S_140001f80_v5_t *)((*((int64_t *)(v4)))));
    v7 = (*((int32_t *)(((int64_t)(v5)))));
    v11 = (v7 & 553648127LL);
    v12 = (v11 == 541541187LL);
    if (v12) {
        v72 = (((int64_t)(v5)) + 4LL);
        v73 = v5->field_4;
        v74 = (v73 & 1LL);
        v75 = (v74 != 0LL);
        if (v75) {
L2:;
            v13 = (v7 > -1073741674LL);
            if (v13) {
L0:;
                v53 = (*((int64_t *)(5368742192LL)));
                v56 = (v53 == 0LL);
                if (v56) {
                    v63 = ((void *)((((int64_t)(v3)) + 32LL)));
                    v64 = (*((int64_t *)(((int64_t)(v63)))));
                    return 0LL;
                } else {
                    v58 = ((void *)((((int64_t)(v3)) + 32LL)));
                    v59 = (*((int64_t *)(((int64_t)(v58)))));
                    /* dac: structuring fallback */
                }
            } else {
                v14 = (v7 <= -1073741685LL);
                if (v14) {
                    v23 = (v7 == -1073741819LL);
                    if (v23) {
                        v44 = ((long long (*)(long long, long long, long long, long long, long long, long long))signal)(v28, v29, 0LL, 11LL, v30, v31);
                        v45 = (v44 == 1LL);
                        if (v45) {
                            v52 = ((long long (*)(long long, long long, long long, long long, long long, long long))signal)(v28, v29, 1LL, 11LL, v30, v31);
L1:;
                            v68 = ((void *)((((int64_t)(v3)) + 32LL)));
                            v69 = (*((int64_t *)(((int64_t)(v68)))));
                            return -1LL;
                        } else {
                            v47 = (v44 == 0LL);
                            if (v47) {
                                goto L0;
                            } else {
                                v49 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v28, v29, 0LL, 11LL, v30, v31);
                                goto L1;
                            }
                        }
                    } else {
                        /* dac: structuring fallback */
                    }
                } else {
                    v15 = (v7 + 1073741683LL);
                    v16 = (v15 > 9LL);
                    if (v16) {
                        goto L1;
                    } else {
                        v18 = (v15 * 4LL);
                        v19 = (5368726408LL + v18);
                        v20 = (*((int32_t *)(v19)));
                        /* recovered switch table at block 4 */
                        switch (v15) {
                            case 5LL: {
                                goto L3;
                            }
                            case 9LL: {
                                goto L4;
                            }
                            default: {
                                /* dac: structuring fallback */
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
/* convention: ms-x64 (score 0.10) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 4 */
/* struct_layouts: pointer=1 stack=1 */
/* switch_tables: 0 */
void __mingwthr_run_key_dtors_part_0(void) {
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
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    int8_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v21 = 0LL;
    int64_t v23 = 0LL;
    S_140002140_v24_t * v24 = ((S_140002140_v24_t *)(0LL));
    int32_t v25 = 0LL;
    int64_t v27 = 0LL;
    int64_t v29 = 0LL;
    int8_t v31 = 0LL;
    int8_t v33 = 0LL;
    int64_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int8_t v43 = 0LL;
    void * v45 = ((void *)(0LL));
    int64_t v46 = 0LL;
    void * v47 = ((void *)(0LL));
    int64_t v49 = 0LL;
    void * v50 = ((void *)(0LL));
    int64_t v52 = 0LL;
    void * v53 = ((void *)(0LL));
    int64_t v55 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 8LL)));
    *((int64_t *)(((int64_t)(v5)))) = v6;
    v7 = ((void *)((((int64_t)(v5)) - 8LL)));
    *((int64_t *)(((int64_t)(v7)))) = v8;
    v9 = ((void *)((((int64_t)(v7)) - 40LL)));
    v14 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v4, v6, v11, 5368742240LL, v12, v13);
    v15 = (*((int64_t *)(5368742208LL)));
    v18 = (v15 == 0LL);
    if (v18) {
    } else {
        v19 = (*((int64_t *)(5368746728LL)));
        v21 = (*((int64_t *)(5368746656LL)));
        while (1) {
            /* phi v23 <- (bb2: v6) (bb8: v27) */
            /* phi v24 <- (bb2: v15) (bb8: v40) */
            v25 = (*((int32_t *)(((int64_t)(v24)))));
            v27 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v21, v23, v11, v25, v12, v13);
            v29 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v21, v27, v11, v25, v12, v13);
            v31 = (v27 == 0LL);
            if (v31) {
            } else {
                v33 = (v29 != 0LL);
                if (v33) {
                } else {
                    v34 = (((int64_t)(v24)) + 8LL);
                    v35 = v24->field_8;
                    v38 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v21, v27, v11, v27, v12, v13);
                }
            }
            v39 = (((int64_t)(v24)) + 16LL);
            v40 = v24->field_10;
            v43 = (v40 != 0LL);
            if (v43) {
                continue;
            }
        }
    }
    v45 = ((void *)((((int64_t)(v9)) + 40LL)));
    v46 = (*((int64_t *)(((int64_t)(v45)))));
    v47 = ((void *)((((int64_t)(v45)) + 8LL)));
    v49 = (*((int64_t *)(((int64_t)(v47)))));
    v50 = ((void *)((((int64_t)(v47)) + 8LL)));
    v52 = (*((int64_t *)(((int64_t)(v50)))));
    v53 = ((void *)((((int64_t)(v50)) + 8LL)));
    v55 = (*((int64_t *)(((int64_t)(v53)))));
    /* dac: structuring fallback */
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
    int64_t v4 = arg0;
    int8_t v7 = 0LL;
    int64_t v10 = 0LL;
    void * v12 = ((void *)(0LL));
    int64_t v13 = arg1;
    void * v16 = ((void *)(0LL));
    uint64_t n = 0LL;
    uint64_t size = 0LL;
    int64_t v19 = arg2;
    S_1400021b0_v20_t * v20 = ((S_1400021b0_v20_t *)(0LL));
    int8_t v22 = 0LL;
    void * v23 = ((void *)(0LL));
    int32_t v24 = 0LL;
    void * v26 = ((void *)(0LL));
    int64_t v27 = 0LL;
    void * v29 = ((void *)(0LL));
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    void * v35 = ((void *)(0LL));
    int64_t v36 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;

    v1 = ((S_1400021b0_v1_t *)((v0 - 56LL)));
    v2 = (*((int32_t *)(5368742216LL)));
    v7 = (v2 != 0LL);
    if (v7) {
        v12 = ((void *)((((int64_t)(v1)) + 72LL)));
        v1->field_48 = v13;
        v16 = ((void *)((((int64_t)(v1)) + 64LL)));
        v1->field_40 = v4;
        v20 = ((S_1400021b0_v20_t *)(((long long (*)(long long, long long, long long, long long, long long, long long))calloc)(n, size, 24LL, 1LL, v19, v4)));
        v22 = (((int64_t)(v20)) == 0LL);
        if (v22) {
        } else {
            v23 = ((void *)((((int64_t)(v1)) + 64LL)));
            v24 = v1->field_40;
            v26 = ((void *)((((int64_t)(v1)) + 72LL)));
            v27 = v1->field_48;
            v29 = ((void *)((((int64_t)(v1)) + 40LL)));
            v1->field_28 = ((int64_t)(v20));
            *((int32_t *)(((int64_t)(v20)))) = v24;
            v31 = (((int64_t)(v20)) + 8LL);
            v20->field_8 = v27;
            v32 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(n, size, 24LL, 5368742240LL, v27, v24);
            v33 = (*((int64_t *)(5368742208LL)));
            v35 = ((void *)((((int64_t)(v1)) + 40LL)));
            v36 = v1->field_28;
            v39 = (v36 + 16LL);
            *((int64_t *)(v39)) = v33;
            *((int64_t *)(5368742208LL)) = v36;
            v40 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(n, size, v33, 5368742240LL, v27, v24);
        }
    } else {
L0:;
        goto L0;
    }
    /* phi v10 <- (bb1: 0) (bb9: -1) */
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
    int8_t v5 = 0LL;
    void * v8 = ((void *)(0LL));
    int64_t v9 = arg0;
    int64_t p = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = arg1;
    int64_t v14 = arg2;
    int64_t v15 = arg3;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int8_t v20 = 0LL;
    void * v21 = ((void *)(0LL));
    int32_t v22 = 0LL;
    S_140002240_v25_t * v25 = ((S_140002240_v25_t *)(0LL));
    int64_t v26 = 0LL;
    int32_t v27 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int8_t v34 = 0LL;
    int8_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int64_t v43 = 0LL;

    v1 = (v0 - 40LL);
    v2 = (*((int32_t *)(5368742216LL)));
    v5 = (v2 != 0LL);
    if (v5) {
        v8 = ((void *)((v1 + 48LL)));
        *((int32_t *)(((int64_t)(v8)))) = v9;
        v16 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(p, v12, v13, 5368742240LL, v14, v15);
        v17 = (*((int64_t *)(5368742208LL)));
        v20 = (v17 == 0LL);
        if (v20) {
        } else {
            v21 = ((void *)((v1 + 48LL)));
            v22 = (*((int32_t *)(((int64_t)(v21)))));
            while (1) {
                /* phi v25 <- (bb5: v17) (bb8: v30) */
                /* phi v26 <- (bb5: 0) (bb8: v25) */
                v27 = (*((int32_t *)(((int64_t)(v25)))));
                v29 = (((int64_t)(v25)) + 16LL);
                v30 = v25->field_10;
                /* dac: structuring fallback */
            }
        }
        /* phi v40 <- (bb4: v13) (bb7: v22) (bb12: v22) */
        /* phi v41 <- (bb4: v14) (bb7: v25) (bb12: v26) */
        v43 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(p, v12, v40, 5368742240LL, v41, v15);
        return 0LL;
    } else {
        return 0LL;
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
/* convention: ms-x64 (score 0.85) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 1 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t __mingw_TLScallback(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = arg1;
    int8_t v3 = 0LL;
    int8_t v5 = 0LL;
    int32_t v6 = 0LL;
    int8_t v9 = 0LL;
    int64_t p = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = arg2;
    int64_t v14 = arg3;
    int64_t v15 = 0LL;
    int32_t v16 = 0LL;
    int8_t v19 = 0LL;
    int32_t v20 = 0LL;
    int8_t v22 = 0LL;
    int64_t v23 = 0LL;
    int8_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    void * v32 = ((void *)(0LL));
    int64_t v33 = 0LL;
    void * v34 = ((void *)(0LL));
    int64_t v35 = 0LL;
    int8_t v38 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = arg0;
    int64_t v42 = 0LL;
    int8_t v45 = 0LL;
    int32_t v46 = 0LL;
    int8_t v49 = 0LL;
    int64_t v50 = 0LL;
    int64_t v51 = 0LL;

    v1 = (v0 - 56LL);
    v3 = (v2 == 2LL);
    if (v3) {
        v51 = ((long long (*)(long long, long long, long long, long long, long long, long long))_fpreset)(p, v12, v2, v41, v13, v14);
        return 1LL;
    } else {
        /* dac: structuring fallback */
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
/* convention: ms-x64 (score 0.40) */
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
    S_1400023f0_v2_t * v2 = arg0;
    int16_t v3 = 0LL;
    int8_t v4 = 0LL;
    int64_t v5 = 0LL;
    int32_t v6 = 0LL;
    S_1400023f0_v8_t * v8 = ((S_1400023f0_v8_t *)(0LL));
    int32_t v9 = 0LL;
    int8_t v10 = 0LL;
    void * v12 = ((void *)(0LL));
    int16_t v13 = 0LL;

    v3 = (*((int16_t *)(((int64_t)(v2)))));
    v4 = (v3 != 23117LL);
    if (v4) {
        return 0LL;
    } else {
        v5 = (((int64_t)(v2)) + 60LL);
        v6 = v2->field_3c;
        v8 = ((S_1400023f0_v8_t *)((((int64_t)(v2)) + v6)));
        v9 = (*((int32_t *)(((int64_t)(v8)))));
        v10 = (v9 == 17744LL);
        if (v10) {
            v12 = ((void *)((((int64_t)(v8)) + 24LL)));
            v13 = v8->field_18;
            (/* opaque: sete */ 0);
            return 0LL;
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
    S_140002420_v4_t * v4 = ((S_140002420_v4_t *)(0LL));
    void * v5 = ((void *)(0LL));
    int16_t v6 = 0LL;
    int8_t v9 = 0LL;
    void * v10 = ((void *)(0LL));
    int16_t v11 = 0LL;
    int16_t v13 = 0LL;
    int16_t v14 = 0LL;
    int16_t v15 = 0LL;
    int64_t v17 = 0LL;
    int16_t v18 = 0LL;
    int16_t v20 = 0LL;
    int16_t v21 = 0LL;
    int16_t v22 = 0LL;
    S_140002420_v24_t * v24 = ((S_140002420_v24_t *)(0LL));
    int64_t v25 = 0LL;
    int32_t v26 = 0LL;
    int64_t v29 = arg1;
    int8_t v30 = 0LL;
    int64_t v31 = 0LL;
    int32_t v32 = 0LL;
    int32_t v33 = 0LL;
    int8_t v34 = 0LL;
    int16_t v35 = 0LL;
    int8_t v36 = 0LL;
    int64_t v39 = 0LL;

    v1 = (v0 + 60LL);
    v2 = (*((int32_t *)(v1)));
    v4 = ((S_140002420_v4_t *)((v2 + v0)));
    v5 = ((void *)((((int64_t)(v4)) + 6LL)));
    v6 = v4->field_6;
    v9 = (v6 == 0LL);
    if (v9) {
    } else {
        v10 = ((void *)((((int64_t)(v4)) + 20LL)));
        v11 = v4->field_14;
        v13 = (v6 - 1LL);
        v14 = (v13 * 4LL);
        v15 = (v13 + v14);
        v17 = (((int64_t)(v4)) + 24LL);
        v18 = (v17 + v11);
        v20 = (v18 + 40LL);
        v21 = (v15 * 8LL);
        v22 = (v20 + v21);
        while (1) {
            /* phi v24 <- (bb1: v18) (bb4: v35) */
            v25 = (((int64_t)(v24)) + 12LL);
            v26 = v24->field_c;
            v30 = (v29 < v26);
            if (v30) {
L1:;
                v35 = (((int64_t)(v24)) + 40LL);
                v36 = (v35 != v22);
                if (v36) {
                    continue;
                } else {
L0:;
                    goto L0;
                }
            } else {
                v31 = (((int64_t)(v24)) + 8LL);
                v32 = v24->field_8;
                v33 = (v26 + v32);
                v34 = (v29 < v33);
                if (v34) {
                } else {
                    goto L1;
                }
            }
        }
    }
    /* phi v39 <- (bb3: v24) (bb5: 0) */
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
/* convention: ms-x64 (score 0.85) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 4 */
/* struct_layouts: pointer=2 stack=1 */
/* switch_tables: 0 */
int64_t _FindPESectionByName(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
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
    int64_t v10 = arg0;
    int64_t v12 = arg1;
    int64_t v13 = arg2;
    int64_t v14 = arg3;
    uint64_t v15 = 0LL;
    int8_t v16 = 0LL;
    S_140002470_v17_t * v17 = ((S_140002470_v17_t *)(0LL));
    int16_t v19 = 0LL;
    int8_t v20 = 0LL;
    int64_t v21 = 0LL;
    int32_t v22 = 0LL;
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
    void * v37 = ((void *)(0LL));
    int64_t v38 = 0LL;
    int64_t i = 0LL;
    int64_t v41 = 0LL;
    int64_t v45 = 0LL;
    int8_t v47 = 0LL;
    void * v48 = ((void *)(0LL));
    int16_t v49 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int8_t v53 = 0LL;
    int64_t v56 = 0LL;
    void * v58 = ((void *)(0LL));
    int64_t v59 = 0LL;
    void * v60 = ((void *)(0LL));
    int64_t v62 = 0LL;
    void * v63 = ((void *)(0LL));
    int64_t v65 = 0LL;
    void * v66 = ((void *)(0LL));
    int64_t v68 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 8LL)));
    *((int64_t *)(((int64_t)(v5)))) = v6;
    v7 = ((void *)((((int64_t)(v5)) - 8LL)));
    *((int64_t *)(((int64_t)(v7)))) = v8;
    v9 = ((void *)((((int64_t)(v7)) - 40LL)));
    v15 = ((long long (*)(long long, long long, long long, long long, long long, long long))strlen)(v10, v6, v12, v10, v13, v14);
    v16 = (v15 > 8LL);
    if (v16) {
    } else {
        v17 = ((S_140002470_v17_t *)((*((int64_t *)(5368726528LL)))));
        v19 = (*((int16_t *)(((int64_t)(v17)))));
        v20 = (v19 == 23117LL);
        if (v20) {
            v21 = (((int64_t)(v17)) + 60LL);
            v22 = v17->field_3c;
            v24 = ((S_140002470_v24_t *)((v22 + ((int64_t)(v17)))));
            v25 = (*((int32_t *)(((int64_t)(v24)))));
            v26 = (v25 != 17744LL);
            if (v26) {
L0:;
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
                        v37 = ((void *)((((int64_t)(v24)) + 24LL)));
                        v38 = (((int64_t)(v37)) + v34);
                        while (1) {
                            /* phi i <- (bb9: 0) (bb12: v51) */
                            /* phi v41 <- (bb9: v38) (bb12: v52) */
                            v45 = ((long long (*)(long long, long long, long long, long long, long long, long long))strncmp)(v10, i, v10, v41, 8LL, v14);
                            v47 = (v45 == 0LL);
                            if (v47) {
                            } else {
                                v48 = ((void *)((((int64_t)(v24)) + 6LL)));
                                v49 = v24->field_6;
                                v51 = (i + 1LL);
                                v52 = (v41 + 40LL);
                                v53 = (v51 < v49);
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
    /* phi v56 <- (bb3: 0) (bb11: v41) */
    v58 = ((void *)((((int64_t)(v9)) + 40LL)));
    v59 = (*((int64_t *)(((int64_t)(v58)))));
    v60 = ((void *)((((int64_t)(v58)) + 8LL)));
    v62 = (*((int64_t *)(((int64_t)(v60)))));
    v63 = ((void *)((((int64_t)(v60)) + 8LL)));
    v65 = (*((int64_t *)(((int64_t)(v63)))));
    v66 = ((void *)((((int64_t)(v63)) + 8LL)));
    v68 = (*((int64_t *)(((int64_t)(v66)))));
    return v56;
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
    S_140002510_v0_t * v0 = ((S_140002510_v0_t *)(0LL));
    int64_t v2 = 0LL;
    int16_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int32_t v7 = 0LL;
    S_140002510_v9_t * v9 = ((S_140002510_v9_t *)(0LL));
    int32_t v10 = 0LL;
    int8_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int16_t v13 = 0LL;
    int8_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int16_t v16 = 0LL;
    int8_t v19 = 0LL;
    void * v20 = ((void *)(0LL));
    int16_t v21 = 0LL;
    int64_t v23 = arg0;
    int64_t v24 = 0LL;
    int16_t v25 = 0LL;
    int16_t v27 = 0LL;
    int16_t v28 = 0LL;
    void * v30 = ((void *)(0LL));
    int64_t v31 = 0LL;
    int64_t v33 = 0LL;
    int16_t v34 = 0LL;
    int16_t v35 = 0LL;
    S_140002510_v37_t * v37 = ((S_140002510_v37_t *)(0LL));
    void * v38 = ((void *)(0LL));
    int32_t v39 = 0LL;
    int8_t v42 = 0LL;
    void * v43 = ((void *)(0LL));
    int32_t v44 = 0LL;
    int32_t v45 = 0LL;
    int8_t v46 = 0LL;
    int64_t v47 = 0LL;
    int8_t v48 = 0LL;
    int64_t v50 = 0LL;

    v0 = ((S_140002510_v0_t *)((*((int64_t *)(5368726528LL)))));
    v4 = (*((int16_t *)(((int64_t)(v0)))));
    v5 = (v4 != 23117LL);
    if (v5) {
L0:;
        /* phi v50 <- (bb0: 0) (bb1: 0) (bb4: 0) (bb5: 0) (bb8: v37) */
        return v50;
    } else {
        v6 = (((int64_t)(v0)) + 60LL);
        v7 = v0->field_3c;
        v9 = ((S_140002510_v9_t *)((v7 + ((int64_t)(v0)))));
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
                v19 = (v16 == 0LL);
                if (v19) {
                    goto L0;
                } else {
                    v20 = ((void *)((((int64_t)(v9)) + 20LL)));
                    v21 = v9->field_14;
                    v24 = (v23 - ((int64_t)(v0)));
                    v25 = (v16 + -1LL);
                    v27 = (v25 * 4LL);
                    v28 = (v25 + v27);
                    v30 = ((void *)((((int64_t)(v9)) + 24LL)));
                    v31 = (((int64_t)(v30)) + v21);
                    v33 = (v31 + 40LL);
                    v34 = (v28 * 8LL);
                    v35 = (v33 + v34);
                    while (1) {
                        /* phi v37 <- (bb6: v31) (bb9: v47) */
                        v38 = ((void *)((((int64_t)(v37)) + 12LL)));
                        v39 = v37->field_c;
                        v42 = (v24 < v39);
                        if (v42) {
L1:;
                            v47 = (((int64_t)(v37)) + 40LL);
                            v48 = (v47 != v35);
                            if (v48) {
                                continue;
                            } else {
                                return 0LL;
                            }
                        } else {
                            v43 = ((void *)((((int64_t)(v37)) + 8LL)));
                            v44 = v37->field_8;
                            v45 = (v39 + v44);
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
int16_t __mingw_GetSectionCount(int64_t arg0) {
    S_140002590_v0_t * v0 = ((S_140002590_v0_t *)(0LL));
    int64_t v2 = arg0;
    int16_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int32_t v7 = 0LL;
    S_140002590_v9_t * v9 = ((S_140002590_v9_t *)(0LL));
    int32_t v10 = 0LL;
    int8_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int16_t v13 = 0LL;
    int8_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int16_t v16 = 0LL;

    v0 = ((S_140002590_v0_t *)((*((int64_t *)(5368726528LL)))));
    v4 = (*((int16_t *)(((int64_t)(v0)))));
    v5 = (v4 != 23117LL);
    if (v5) {
        return ((int16_t)(0LL));
    } else {
        v6 = (((int64_t)(v0)) + 60LL);
        v7 = v0->field_3c;
        v9 = ((S_140002590_v9_t *)((((int64_t)(v0)) + v7)));
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
                return v16;
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
    S_1400025d0_v0_t * v0 = ((S_1400025d0_v0_t *)(0LL));
    int64_t v2 = 0LL;
    int16_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int32_t v7 = 0LL;
    S_1400025d0_v9_t * v9 = ((S_1400025d0_v9_t *)(0LL));
    int32_t v10 = 0LL;
    int8_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int16_t v13 = 0LL;
    int8_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int16_t v16 = 0LL;
    int8_t v19 = 0LL;
    void * v20 = ((void *)(0LL));
    int16_t v21 = 0LL;
    void * v23 = ((void *)(0LL));
    int64_t v24 = 0LL;
    int16_t v26 = 0LL;
    int16_t v28 = 0LL;
    int16_t v29 = 0LL;
    int64_t v31 = 0LL;
    int16_t v32 = 0LL;
    int16_t v33 = 0LL;
    int64_t v35 = arg0;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    void * v38 = ((void *)(0LL));
    int8_t v39 = 0LL;
    int8_t v40 = 0LL;
    int8_t v41 = 0LL;
    int8_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    int8_t v47 = 0LL;
    int64_t v49 = 0LL;

    v0 = ((S_1400025d0_v0_t *)((*((int64_t *)(5368726528LL)))));
    v4 = (*((int16_t *)(((int64_t)(v0)))));
    v5 = (v4 != 23117LL);
    if (v5) {
L0:;
        /* phi v49 <- (bb0: 0) (bb1: 0) (bb4: 0) (bb5: 0) (bb8: v36) */
        return v49;
    } else {
        v6 = (((int64_t)(v0)) + 60LL);
        v7 = v0->field_3c;
        v9 = ((S_1400025d0_v9_t *)((v7 + ((int64_t)(v0)))));
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
                v19 = (v16 == 0LL);
                if (v19) {
                    goto L0;
                } else {
                    v20 = ((void *)((((int64_t)(v9)) + 20LL)));
                    v21 = v9->field_14;
                    v23 = ((void *)((((int64_t)(v9)) + 24LL)));
                    v24 = (((int64_t)(v23)) + v21);
                    v26 = (v16 + -1LL);
                    v28 = (v26 * 4LL);
                    v29 = (v26 + v28);
                    v31 = (v24 + 40LL);
                    v32 = (v29 * 8LL);
                    v33 = (v31 + v32);
                    while (1) {
                        /* phi v36 <- (bb6: v24) (bb10: v46) */
                        /* phi v37 <- (bb6: v35) (bb10: v45) */
                        v38 = ((void *)((v36 + 39LL)));
                        v39 = (*((int8_t *)(((int64_t)(v38)))));
                        v40 = (v39 & 32LL);
                        v41 = (v40 == 0LL);
                        if (v41) {
L1:;
                            /* phi v45 <- (bb7: v37) (bb9: v44) */
                            v46 = (v36 + 40LL);
                            v47 = (v33 != v46);
                            if (v47) {
                                continue;
                            } else {
                                return 0LL;
                            }
                        } else {
                            v43 = (v37 == 0LL);
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
/* convention: ms-x64 (score 0.45) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=2 stack=0 */
/* switch_tables: 0 */
int64_t _GetPEImageBase(void) {
    S_140002650_v0_t * v0 = ((S_140002650_v0_t *)(0LL));
    int64_t v2 = 0LL;
    int16_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int32_t v7 = 0LL;
    S_140002650_v9_t * v9 = ((S_140002650_v9_t *)(0LL));
    int32_t v10 = 0LL;
    int8_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int16_t v13 = 0LL;

    v0 = ((S_140002650_v0_t *)((*((int64_t *)(5368726528LL)))));
    v4 = (*((int16_t *)(((int64_t)(v0)))));
    v5 = (v4 != 23117LL);
    if (v5) {
        return 0LL;
    } else {
        v6 = (((int64_t)(v0)) + 60LL);
        v7 = v0->field_3c;
        v9 = ((S_140002650_v9_t *)((v7 + ((int64_t)(v0)))));
        v10 = (*((int32_t *)(((int64_t)(v9)))));
        v11 = (v10 == 17744LL);
        if (v11) {
            v12 = ((void *)((((int64_t)(v9)) + 24LL)));
            v13 = v9->field_18;
            (/* opaque: cmove */ 0);
            return 0LL;
        } else {
L0:;
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
    S_140002690_v0_t * v0 = ((S_140002690_v0_t *)(0LL));
    int64_t v2 = 0LL;
    int16_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int32_t v7 = 0LL;
    S_140002690_v9_t * v9 = ((S_140002690_v9_t *)(0LL));
    int32_t v10 = 0LL;
    int8_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int16_t v13 = 0LL;
    int8_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int16_t v16 = 0LL;
    int8_t v19 = 0LL;
    void * v20 = ((void *)(0LL));
    int16_t v21 = 0LL;
    int64_t v23 = arg0;
    int64_t v24 = 0LL;
    int16_t v25 = 0LL;
    int16_t v27 = 0LL;
    int16_t v28 = 0LL;
    void * v30 = ((void *)(0LL));
    int64_t v31 = 0LL;
    int64_t v33 = 0LL;
    int16_t v34 = 0LL;
    int16_t v35 = 0LL;
    S_140002690_v37_t * v37 = ((S_140002690_v37_t *)(0LL));
    void * v38 = ((void *)(0LL));
    int32_t v39 = 0LL;
    int8_t v42 = 0LL;
    void * v43 = ((void *)(0LL));
    int32_t v44 = 0LL;
    int32_t v45 = 0LL;
    int8_t v46 = 0LL;
    void * v47 = ((void *)(0LL));
    int32_t v48 = 0LL;
    int32_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int8_t v53 = 0LL;

    v0 = ((S_140002690_v0_t *)((*((int64_t *)(5368726528LL)))));
    v4 = (*((int16_t *)(((int64_t)(v0)))));
    v5 = (v4 != 23117LL);
    if (v5) {
        return 0LL;
    } else {
        v6 = (((int64_t)(v0)) + 60LL);
        v7 = v0->field_3c;
        v9 = ((S_140002690_v9_t *)((v7 + ((int64_t)(v0)))));
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
                v19 = (v16 == 0LL);
                if (v19) {
                    goto L0;
                } else {
                    v20 = ((void *)((((int64_t)(v9)) + 20LL)));
                    v21 = v9->field_14;
                    v24 = (v23 - ((int64_t)(v0)));
                    v25 = (v16 + -1LL);
                    v27 = (v25 * 4LL);
                    v28 = (v25 + v27);
                    v30 = ((void *)((((int64_t)(v9)) + 24LL)));
                    v31 = (((int64_t)(v30)) + v21);
                    v33 = (v31 + 40LL);
                    v34 = (v28 * 8LL);
                    v35 = (v33 + v34);
                    while (1) {
                        /* phi v37 <- (bb6: v31) (bb9: v52) */
                        v38 = ((void *)((((int64_t)(v37)) + 12LL)));
                        v39 = v37->field_c;
                        v42 = (v24 < v39);
                        if (v42) {
L1:;
                            v52 = (((int64_t)(v37)) + 40LL);
                            v53 = (v35 != v52);
                            if (v53) {
                                continue;
                            } else {
                                break;
                            }
                        } else {
                            v43 = ((void *)((((int64_t)(v37)) + 8LL)));
                            v44 = v37->field_8;
                            v45 = (v39 + v44);
                            v46 = (v24 < v45);
                            if (v46) {
                                v47 = ((void *)((((int64_t)(v37)) + 36LL)));
                                v48 = v37->field_24;
                                v50 = ~(v48);
                                v51 = (v50 >> 31LL);
                                return v51;
                            } else {
                                goto L1;
                            }
                        }
                    }
                    return 0LL;
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
    S_140002720_v0_t * v0 = ((S_140002720_v0_t *)(0LL));
    int64_t v2 = 0LL;
    int16_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int32_t v7 = 0LL;
    S_140002720_v9_t * v9 = ((S_140002720_v9_t *)(0LL));
    int32_t v10 = 0LL;
    int8_t v11 = 0LL;
    void * v12 = ((void *)(0LL));
    int16_t v13 = 0LL;
    int8_t v14 = 0LL;
    void * v15 = ((void *)(0LL));
    int32_t v16 = 0LL;
    int8_t v19 = 0LL;
    void * v20 = ((void *)(0LL));
    int16_t v21 = 0LL;
    int8_t v24 = 0LL;
    void * v25 = ((void *)(0LL));
    int16_t v26 = 0LL;
    void * v28 = ((void *)(0LL));
    int64_t v29 = 0LL;
    int16_t v31 = 0LL;
    int16_t v33 = 0LL;
    int16_t v34 = 0LL;
    int64_t v36 = 0LL;
    int16_t v37 = 0LL;
    int16_t v38 = 0LL;
    S_140002720_v40_t * v40 = ((S_140002720_v40_t *)(0LL));
    void * v41 = ((void *)(0LL));
    int32_t v42 = 0LL;
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
    int8_t v58 = 0LL;
    void * v59 = ((void *)(0LL));
    int32_t v60 = 0LL;
    int8_t v63 = 0LL;
    int8_t v65 = 0LL;
    int64_t v66 = 0LL;
    int64_t v67 = 0LL;
    void * v68 = ((void *)(0LL));
    int32_t v69 = 0LL;
    int64_t v71 = 0LL;
    int64_t v74 = 0LL;
    int8_t v75 = 0LL;

    v0 = ((S_140002720_v0_t *)((*((int64_t *)(5368726528LL)))));
    v4 = (*((int16_t *)(((int64_t)(v0)))));
    v5 = (v4 != 23117LL);
    if (v5) {
        return 0LL;
    } else {
        v6 = (((int64_t)(v0)) + 60LL);
        v7 = v0->field_3c;
        v9 = ((S_140002720_v9_t *)((v7 + ((int64_t)(v0)))));
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
                v15 = ((void *)((((int64_t)(v9)) + 144LL)));
                v16 = v9->field_90;
                v19 = (v16 == 0LL);
                if (v19) {
                    goto L0;
                } else {
                    v20 = ((void *)((((int64_t)(v9)) + 6LL)));
                    v21 = v9->field_6;
                    v24 = (v21 == 0LL);
                    if (v24) {
                        goto L0;
                    } else {
                        v25 = ((void *)((((int64_t)(v9)) + 20LL)));
                        v26 = v9->field_14;
                        v28 = ((void *)((((int64_t)(v9)) + 24LL)));
                        v29 = (((int64_t)(v28)) + v26);
                        v31 = (v21 + -1LL);
                        v33 = (v31 * 4LL);
                        v34 = (v31 + v33);
                        v36 = (v29 + 40LL);
                        v37 = (v34 * 8LL);
                        v38 = (v36 + v37);
                        while (1) {
                            /* phi v40 <- (bb7: v29) (bb10: v74) */
                            v41 = ((void *)((((int64_t)(v40)) + 12LL)));
                            v42 = v40->field_c;
                            v45 = (v16 < v42);
                            if (v45) {
L2:;
                                v74 = (((int64_t)(v40)) + 40LL);
                                v75 = (v38 != v74);
                                if (v75) {
                                    continue;
                                } else {
                                    break;
                                }
                            } else {
                                v46 = ((void *)((((int64_t)(v40)) + 8LL)));
                                v47 = v40->field_8;
                                v48 = (v42 + v47);
                                v49 = (v16 < v48);
                                if (v49) {
                                    v50 = (v16 + ((int64_t)(v0)));
                                    while (1) {
                                        /* phi v52 <- (bb13: v50) (bb15: v67) */
                                        /* phi v53 <- (bb13: v51) (bb15: v66) */
                                        v54 = ((void *)((((int64_t)(v52)) + 4LL)));
                                        v55 = v52->field_4;
                                        v58 = (v55 != 0LL);
                                        if (v58) {
L1:;
                                            v65 = (v53 > 0LL);
                                            if (v65) {
                                                v66 = (v53 - 1LL);
                                                v67 = (((int64_t)(v52)) + 20LL);
                                                continue;
                                            } else {
                                                v68 = ((void *)((((int64_t)(v52)) + 12LL)));
                                                v69 = v52->field_c;
                                                v71 = (v69 + ((int64_t)(v0)));
                                                return v71;
                                            }
                                        } else {
                                            v59 = ((void *)((((int64_t)(v52)) + 12LL)));
                                            v60 = v52->field_c;
                                            v63 = (v60 == 0LL);
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
                        return 0LL;
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
    int64_t v22 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((S_1400027e0_v3_t *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) + 24LL)));
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002820 */
/* end: 0x140002860 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 3 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.85) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 4 */
/* struct_layouts: pointer=1 stack=1 */
/* switch_tables: 0 */
int64_t vfprintf(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    int64_t v6 = 0LL;
    S_140002820_v7_t * v7 = ((S_140002820_v7_t *)(0LL));
    int64_t v8 = arg0;
    int64_t v10 = arg1;
    int64_t v12 = arg2;
    int64_t v14 = arg3;
    void * v15 = ((void *)(0LL));
    int64_t v19 = 0LL;
    void * v21 = ((void *)(0LL));
    int64_t v22 = 0LL;
    void * v23 = ((void *)(0LL));
    int64_t v24 = 0LL;
    void * v25 = ((void *)(0LL));
    int64_t v27 = 0LL;
    void * v28 = ((void *)(0LL));
    int64_t v30 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 8LL)));
    *((int64_t *)(((int64_t)(v5)))) = v6;
    v7 = ((S_140002820_v7_t *)((((int64_t)(v5)) - 48LL)));
    v15 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__local_stdio_printf_options)(v12, v10, v10, v8, v12, v14)));
    v19 = (*((int64_t *)(((int64_t)(v15)))));
    v21 = ((void *)((((int64_t)(v7)) + 32LL)));
    v7->field_20 = v12;
    v22 = ((long long (*)(long long, long long, long long, long long, long long, long long))__stdio_common_vfprintf)(v12, v10, v8, v19, v10, 0LL);
    v23 = ((void *)((((int64_t)(v7)) + 48LL)));
    v24 = v7->field_30;
    v25 = ((void *)((((int64_t)(v23)) + 8LL)));
    v27 = (*((int64_t *)(((int64_t)(v25)))));
    v28 = ((void *)((((int64_t)(v25)) + 8LL)));
    v30 = (*((int64_t *)(((int64_t)(v28)))));
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
    int64_t v10 = arg0;
    int64_t v12 = arg1;
    int64_t v15 = arg2;
    void * v16 = ((void *)(0LL));
    int64_t v17 = arg3;
    void * v18 = ((void *)(0LL));
    void * v19 = ((void *)(0LL));
    int64_t v23 = 0LL;
    void * v25 = ((void *)(0LL));
    int64_t v26 = 0LL;
    void * v27 = ((void *)(0LL));
    int64_t v28 = 0LL;
    void * v29 = ((void *)(0LL));
    int64_t v31 = 0LL;
    void * v32 = ((void *)(0LL));
    int64_t v34 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 8LL)));
    *((int64_t *)(((int64_t)(v5)))) = v6;
    v7 = ((S_140002860_v7_t *)((((int64_t)(v5)) - 64LL)));
    v8 = ((void *)((((int64_t)(v7)) + 112LL)));
    v7->field_70 = v15;
    v16 = ((void *)((((int64_t)(v7)) + 120LL)));
    v7->field_78 = v17;
    v18 = ((void *)((((int64_t)(v7)) + 56LL)));
    v7->field_38 = ((int64_t)(v8));
    v19 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__local_stdio_printf_options)(((int64_t)(v8)), v12, v12, v10, v15, v17)));
    v23 = (*((int64_t *)(((int64_t)(v19)))));
    v25 = ((void *)((((int64_t)(v7)) + 32LL)));
    v7->field_20 = ((int64_t)(v8));
    v26 = ((long long (*)(long long, long long, long long, long long, long long, long long))__stdio_common_vfprintf)(((int64_t)(v8)), v12, v10, v23, v12, 0LL);
    v27 = ((void *)((((int64_t)(v7)) + 64LL)));
    v28 = v7->field_40;
    v29 = ((void *)((((int64_t)(v27)) + 8LL)));
    v31 = (*((int64_t *)(((int64_t)(v29)))));
    v32 = ((void *)((((int64_t)(v29)) + 8LL)));
    v34 = (*((int64_t *)(((int64_t)(v32)))));
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
/* convention: ms-x64 (score 0.55) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t __local_stdio_printf_options(void) {
    return 5368721504LL;
}

/* dac-recovered function */
/* address: 0x1400028c0 */
/* end: 0x1400028d0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.55) */
/* args: (no register args) */
/* return_reg: rax */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t __p___initenv(void) {
    int64_t v0 = 0LL;
    int64_t v2 = 0LL;

    v0 = (*((int64_t *)(5368726608LL)));
    v2 = (*((int64_t *)(v0)));
    return v2;
}

/* dac-recovered function */
/* address: 0x1400028d0 */
/* end: 0x140002900 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 4 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.70) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: none */
/* stack_locals: 1 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void _amsg_exit(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    int64_t v4 = arg0;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = arg1;
    int64_t v10 = arg2;
    int64_t v11 = arg3;
    int64_t v12 = 0LL;
    int64_t v16 = 0LL;
    int64_t v18 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v12 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v7, v8, v9, 2LL, v10, v11);
    v16 = ((long long (*)(long long, long long, long long, long long, long long, long long))fprintf)(v7, v8, 5368726448LL, v12, v4, v11);
    v18 = ((long long (*)(long long, long long, long long, long long, long long, long long))_exit)(v7, v8, 5368726448LL, 255LL, v4, v11);
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002900 */
/* end: 0x140002960 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 7 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.90) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 5 */
/* struct_layouts: pointer=1 stack=1 */
/* switch_tables: 0 */
int64_t __getmainargs(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    void * v1 = ((void *)(0LL));
    int64_t v2 = 0LL;
    void * v3 = ((void *)(0LL));
    int64_t v4 = 0LL;
    void * v5 = ((void *)(0LL));
    int64_t v6 = 0LL;
    void * v7 = ((void *)(0LL));
    int64_t v8 = 0LL;
    S_140002900_v9_t * v9 = ((S_140002900_v9_t *)(0LL));
    int64_t v10 = arg3;
    int64_t v12 = arg1;
    int64_t v14 = arg2;
    int64_t v16 = arg0;
    int64_t v18 = 0LL;
    int64_t v21 = 0LL;
    void * v22 = ((void *)(0LL));
    int32_t v23 = 0LL;
    void * v25 = ((void *)(0LL));
    int64_t v26 = 0LL;
    void * v28 = ((void *)(0LL));
    int64_t v29 = 0LL;
    void * v31 = ((void *)(0LL));
    int64_t v32 = 0LL;
    int32_t v34 = 0LL;
    int64_t v36 = 0LL;
    void * v38 = ((void *)(0LL));
    int64_t v39 = 0LL;
    void * v40 = ((void *)(0LL));
    int64_t v42 = 0LL;
    void * v43 = ((void *)(0LL));
    int64_t v45 = 0LL;
    void * v46 = ((void *)(0LL));
    int64_t v48 = 0LL;

    v1 = ((void *)((v0 - 8LL)));
    *((int64_t *)(((int64_t)(v1)))) = v2;
    v3 = ((void *)((((int64_t)(v1)) - 8LL)));
    *((int64_t *)(((int64_t)(v3)))) = v4;
    v5 = ((void *)((((int64_t)(v3)) - 8LL)));
    *((int64_t *)(((int64_t)(v5)))) = v6;
    v7 = ((void *)((((int64_t)(v5)) - 8LL)));
    *((int64_t *)(((int64_t)(v7)))) = v8;
    v9 = ((S_140002900_v9_t *)((((int64_t)(v7)) - 40LL)));
    v18 = ((long long (*)(long long, long long, long long, long long, long long, long long))_initialize_narrow_environment)(v16, v12, v12, v16, v14, v10);
    v21 = ((long long (*)(long long, long long, long long, long long, long long, long long))_configure_narrow_argv)(v16, v12, v12, 2LL, v14, v10);
    v22 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__p___argc)(v16, v12, v12, 2LL, v14, v10)));
    v23 = (*((int32_t *)(((int64_t)(v22)))));
    *((int32_t *)(v16)) = v23;
    v25 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__p___argv)(v16, v12, v12, 2LL, v14, v10)));
    v26 = (*((int64_t *)(((int64_t)(v25)))));
    *((int64_t *)(v12)) = v26;
    v28 = ((void *)(((long long (*)(long long, long long, long long, long long, long long, long long))__p__environ)(v16, v12, v12, 2LL, v14, v10)));
    v29 = (*((int64_t *)(((int64_t)(v28)))));
    *((int64_t *)(v14)) = v29;
    v31 = ((void *)((((int64_t)(v9)) + 112LL)));
    v32 = v9->field_70;
    v34 = (*((int32_t *)(v32)));
    v36 = ((long long (*)(long long, long long, long long, long long, long long, long long))_set_new_mode)(v16, v12, v12, v34, v14, v10);
    v38 = ((void *)((((int64_t)(v9)) + 40LL)));
    v39 = v9->field_28;
    v40 = ((void *)((((int64_t)(v38)) + 8LL)));
    v42 = (*((int64_t *)(((int64_t)(v40)))));
    v43 = ((void *)((((int64_t)(v40)) + 8LL)));
    v45 = (*((int64_t *)(((int64_t)(v43)))));
    v46 = ((void *)((((int64_t)(v43)) + 8LL)));
    v48 = (*((int64_t *)(((int64_t)(v46)))));
    return 0LL;
}

/* dac-recovered function */
/* address: 0x140002960 */
/* end: 0x140002968 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void strlen(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002968 */
/* end: 0x140002970 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void strncmp(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002970 */
/* end: 0x140002978 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void __acrt_iob_func(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002978 */
/* end: 0x140002980 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void __p__commode(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002980 */
/* end: 0x140002988 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void __p__fmode(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002988 */
/* end: 0x140002990 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void __stdio_common_vfprintf(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002990 */
/* end: 0x140002998 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void fflush(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002998 */
/* end: 0x1400029a0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void setvbuf(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x1400029a0 */
/* end: 0x1400029a8 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void __set_app_type(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x1400029a8 */
/* end: 0x1400029b0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void __p___argc(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x1400029b0 */
/* end: 0x1400029b8 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void __p___argv(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x1400029b8 */
/* end: 0x1400029c0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void _cexit(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x1400029c0 */
/* end: 0x1400029c8 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void _configure_narrow_argv(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x1400029c8 */
/* end: 0x1400029d0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void _crt_atexit(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x1400029d0 */
/* end: 0x1400029d8 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void _exit(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x1400029d8 */
/* end: 0x1400029e0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void _initialize_narrow_environment(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x1400029e0 */
/* end: 0x1400029e8 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void _initterm(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x1400029e8 */
/* end: 0x1400029f0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void _initterm_e(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x1400029f0 */
/* end: 0x1400029f8 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void _set_invalid_parameter_handler(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x1400029f8 */
/* end: 0x140002a00 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void abort(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002a00 */
/* end: 0x140002a08 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void exit(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002a08 */
/* end: 0x140002a18 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void signal(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002a18 */
/* end: 0x140002a20 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void memcpy(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002a20 */
/* end: 0x140002a28 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void __setusermatherr(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002a28 */
/* end: 0x140002a30 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void _configthreadlocale(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002a30 */
/* end: 0x140002a38 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void _set_new_mode(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002a38 */
/* end: 0x140002a40 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void calloc(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002a40 */
/* end: 0x140002a48 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void free(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002a48 */
/* end: 0x140002a50 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void malloc(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002a50 */
/* end: 0x140002a60 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 1 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.40) */
/* args: (no register args) */
/* return_reg: none */
/* stack_locals: 0 */
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
void __p__environ(void) {
    /* dac: structuring fallback */
}

/* dac-recovered function */
/* address: 0x140002a60 */
/* end: 0x140002ac0 */
/* confidence: 1.00 (Observed) */
/* source_blocks: 4 */
/* goto_count: 0 */
/* label_count: 0 */
/* irreducible: false */
/* convention: ms-x64 (score 0.85) */
/* args: rcx,rdx,r8,r9 */
/* return_reg: rax */
/* stack_locals: 3 */
/* struct_layouts: pointer=1 stack=0 */
/* switch_tables: 0 */
int64_t main(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    S_140002a60_v1_t * v1 = ((S_140002a60_v1_t *)(0LL));
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = arg1;
    int64_t v5 = arg0;
    int64_t v6 = arg2;
    int64_t v7 = arg3;
    int64_t v8 = 0LL;
    int64_t v10 = 0LL;
    void * v13 = ((void *)(0LL));
    int64_t v16 = 0LL;
    void * v19 = ((void *)(0LL));
    void * v21 = ((void *)(0LL));
    int64_t v22 = 0LL;

    v1 = ((S_140002a60_v1_t *)((v0 - 88LL)));
    v8 = ((long long (*)(long long, long long, long long, long long, long long, long long))__main)(v2, v3, v4, v5, v6, v7);
    v10 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v2, v3, v4, -11LL, v6, v7);
    (/* opaque: movaps */ 0);
    v13 = ((void *)((((int64_t)(v1)) + 56LL)));
    v1->field_38 = 0LL;
    v16 = (((int64_t)(v1)) + 61LL);
    v19 = ((void *)((((int64_t)(v1)) + 32LL)));
    v1->field_20 = 0LL;
    (/* opaque: movups */ 0);
    v21 = ((void *)((((int64_t)(v1)) + 76LL)));
    v1->field_4c = 673104LL;
    v22 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v2, v3, v16, v10, 18LL, ((int64_t)(v13)));
    return 42LL;
}

/* dac-recovered forwarding thunk */
/* address: 0x140002ac0 */
/* end: 0x140002ad0 */
/* confidence: 1.00 (Observed) */
/* tail-call: __gcc_register_frame (0x140001490) */
void register_frame_ctor(void) {
    __gcc_register_frame();
}
