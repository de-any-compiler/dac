/* dac --target c -O1 reconstruction
   input: tests/fixtures/hello-x86_64.exe
   arch:  x86-64 */
#include <stdint.h>
#include <stddef.h>

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
/* struct_layouts: pointer=4 stack=1 */
/* switch_tables: 0 */
int64_t __tmainCRTStartup(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
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
    int64_t v12 = arg0;
    int64_t v13 = 0LL;
    int64_t v14 = arg1;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
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
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = arg3;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int8_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = arg2;
    int64_t v37 = arg4;
    int64_t v38 = arg5;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int32_t v46 = 0LL;
    int64_t v47 = 0LL;
    int8_t v48 = 0LL;
    int32_t v49 = 0LL;
    int64_t v50 = 0LL;
    int64_t v51 = 0LL;
    int8_t v52 = 0LL;
    int64_t v53 = 0LL;
    int64_t v54 = 0LL;
    int64_t v55 = 0LL;
    int64_t v56 = 0LL;
    int64_t v57 = 0LL;
    int64_t v58 = 0LL;
    int64_t v59 = 0LL;
    int64_t v60 = 0LL;
    int64_t v61 = 0LL;
    int8_t v62 = 0LL;
    int64_t v63 = 0LL;
    int64_t v64 = 0LL;
    int64_t v65 = 0LL;
    int64_t v66 = 0LL;
    int64_t v67 = 0LL;
    int64_t v68 = 0LL;
    int8_t v69 = 0LL;
    int64_t v70 = 0LL;
    int64_t v71 = 0LL;
    int64_t v72 = 0LL;
    int64_t v73 = 0LL;
    int64_t v74 = 0LL;
    int64_t v75 = 0LL;
    int64_t v76 = 0LL;
    int64_t v77 = 0LL;
    int64_t v78 = 0LL;
    int64_t v79 = 0LL;
    int64_t v80 = 0LL;
    int64_t v81 = 0LL;
    int32_t v82 = 0LL;
    int64_t v83 = 0LL;
    int64_t v84 = 0LL;
    int64_t v85 = 0LL;
    int64_t v86 = 0LL;
    int64_t v87 = 0LL;
    int64_t v88 = 0LL;
    int32_t v89 = 0LL;
    int64_t v90 = 0LL;
    int64_t v91 = 0LL;
    int8_t v92 = 0LL;
    int64_t v93 = 0LL;
    int32_t v94 = 0LL;
    int64_t v95 = 0LL;
    int64_t v96 = 0LL;
    int8_t v97 = 0LL;
    int64_t v98 = 0LL;
    int64_t v99 = 0LL;
    int64_t v100 = 0LL;
    int64_t v101 = 0LL;
    int64_t v102 = 0LL;
    int64_t v103 = 0LL;
    int64_t v104 = 0LL;
    int64_t v105 = 0LL;
    int64_t v106 = 0LL;
    int64_t v107 = 0LL;
    int64_t v108 = 0LL;
    int64_t v109 = 0LL;
    int64_t v110 = 0LL;
    int64_t v111 = 0LL;
    int64_t v112 = 0LL;
    int64_t v113 = 0LL;
    int64_t v114 = 0LL;
    int64_t v115 = 0LL;
    int64_t v116 = 0LL;
    int64_t v117 = 0LL;
    int64_t v118 = 0LL;
    int64_t v119 = 0LL;
    int64_t v120 = 0LL;
    int64_t v121 = 0LL;
    int64_t v122 = 0LL;
    int64_t v123 = 0LL;
    int64_t v124 = 0LL;
    int64_t v125 = 0LL;
    int64_t v126 = 0LL;
    int32_t v127 = 0LL;
    int64_t v128 = 0LL;
    int64_t v129 = 0LL;
    int64_t v130 = 0LL;
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
    int8_t v142 = 0LL;
    int64_t v143 = 0LL;
    int64_t v144 = 0LL;
    int64_t v145 = 0LL;
    int64_t v146 = 0LL;
    int64_t v147 = 0LL;
    int64_t v148 = 0LL;
    int64_t v149 = 0LL;
    int64_t v150 = 0LL;
    int64_t v151 = 0LL;
    int64_t v152 = 0LL;
    int64_t v153 = 0LL;
    int64_t v154 = 0LL;
    int64_t v155 = 0LL;
    int64_t v156 = 0LL;
    int64_t v157 = 0LL;
    int64_t v158 = 0LL;
    int64_t v159 = 0LL;
    int64_t v160 = 0LL;
    int64_t v161 = 0LL;
    int64_t v162 = 0LL;
    int64_t v163 = 0LL;
    int64_t v164 = 0LL;
    int64_t v165 = 0LL;
    int16_t v166 = 0LL;
    int8_t v167 = 0LL;
    int64_t v168 = 0LL;
    int32_t v169 = 0LL;
    int64_t v170 = 0LL;
    int64_t v171 = 0LL;
    int32_t v172 = 0LL;
    int8_t v173 = 0LL;
    int64_t v174 = 0LL;
    int16_t v175 = 0LL;
    int64_t v176 = 0LL;
    int8_t v177 = 0LL;
    int8_t v178 = 0LL;
    int64_t v179 = 0LL;
    int32_t v180 = 0LL;
    int8_t v181 = 0LL;
    int64_t v182 = 0LL;
    int32_t v183 = 0LL;
    int64_t v184 = 0LL;
    int64_t v185 = 0LL;
    int64_t v186 = 0LL;
    int32_t v187 = 0LL;
    int8_t v188 = 0LL;
    int64_t v189 = 0LL;
    int32_t v190 = 0LL;
    int64_t v191 = 0LL;
    int64_t v192 = 0LL;
    int64_t v193 = 0LL;
    int64_t v194 = 0LL;
    int64_t v195 = 0LL;
    int64_t v196 = 0LL;
    int64_t v197 = 0LL;
    int64_t v198 = 0LL;
    int64_t v199 = 0LL;
    int32_t v200 = 0LL;
    int64_t v201 = 0LL;
    int64_t v202 = 0LL;
    int8_t v203 = 0LL;
    int64_t v204 = 0LL;
    int64_t v205 = 0LL;
    int64_t n = 0LL;
    int64_t v207 = 0LL;
    int64_t v208 = 0LL;
    int64_t v209 = 0LL;
    int64_t v210 = 0LL;
    int64_t v211 = 0LL;
    int64_t v212 = 0LL;
    int64_t v213 = 0LL;
    int64_t v214 = 0LL;
    int64_t v215 = 0LL;
    int32_t v216 = 0LL;
    int64_t v217 = 0LL;
    int64_t v218 = 0LL;
    int64_t v219 = 0LL;
    int64_t v220 = 0LL;
    int64_t v221 = 0LL;
    int32_t v222 = 0LL;
    int64_t v223 = 0LL;
    int64_t v224 = 0LL;
    int64_t v225 = 0LL;
    int64_t v226 = 0LL;
    int64_t v227 = 0LL;
    int32_t v228 = 0LL;
    int8_t v229 = 0LL;
    int64_t v230 = 0LL;
    int64_t v231 = 0LL;
    int64_t v232 = 0LL;
    int32_t v233 = 0LL;
    int8_t v234 = 0LL;
    int64_t v235 = 0LL;
    int64_t v236 = 0LL;
    int64_t v237 = 0LL;
    int64_t v238 = 0LL;
    int64_t v239 = 0LL;
    int64_t v240 = 0LL;
    int64_t v241 = 0LL;
    int64_t v242 = 0LL;
    int8_t v243 = 0LL;
    int64_t v244 = 0LL;
    int64_t v245 = 0LL;
    int64_t v246 = 0LL;
    int64_t v247 = 0LL;
    int64_t v248 = 0LL;
    int64_t v249 = 0LL;
    int64_t v250 = 0LL;
    int64_t v251 = 0LL;
    int64_t v252 = 0LL;
    int32_t v253 = 0LL;
    int64_t v254 = 0LL;
    int64_t v255 = 0LL;
    int64_t v256 = 0LL;
    int64_t v257 = 0LL;
    int64_t v258 = 0LL;
    int32_t v259 = 0LL;
    int64_t v260 = 0LL;
    int64_t v261 = 0LL;
    int64_t v262 = 0LL;
    int64_t v263 = 0LL;
    int64_t v264 = 0LL;
    int64_t v265 = 0LL;
    int32_t v266 = 0LL;
    int64_t v267 = 0LL;
    int64_t v268 = 0LL;
    int64_t v269 = 0LL;
    int64_t v270 = 0LL;
    int64_t v271 = 0LL;
    int64_t v272 = 0LL;
    int64_t v273 = 0LL;
    int64_t v274 = 0LL;
    int8_t v275 = 0LL;
    int64_t v276 = 0LL;
    int8_t v277 = 0LL;
    int64_t v278 = 0LL;
    int64_t v279 = 0LL;
    int64_t v280 = 0LL;
    int64_t v281 = 0LL;
    int64_t s = 0LL;
    int64_t src = 0LL;
    int64_t v284 = 0LL;
    int64_t v285 = 0LL;
    int64_t v286 = 0LL;
    int64_t v287 = 0LL;
    int64_t v288 = 0LL;
    int64_t v289 = 0LL;
    int64_t v290 = 0LL;
    int64_t v291 = 0LL;
    int64_t v292 = 0LL;
    int64_t dst = 0LL;
    int64_t v294 = 0LL;
    int64_t v295 = 0LL;
    int64_t v296 = 0LL;
    int64_t v297 = 0LL;
    int64_t v298 = 0LL;
    int64_t v299 = 0LL;
    int8_t v300 = 0LL;
    int64_t v301 = 0LL;
    int64_t v302 = 0LL;
    int64_t v303 = 0LL;
    int64_t v304 = 0LL;
    int64_t n_1 = 0LL;
    int64_t v306 = 0LL;
    int64_t v307 = 0LL;
    int64_t v308 = 0LL;
    int8_t v309 = 0LL;
    int64_t v310 = 0LL;
    int64_t v311 = 0LL;
    int64_t v312 = 0LL;
    int64_t v313 = 0LL;
    int64_t v314 = 0LL;
    int64_t v315 = 0LL;
    int64_t v316 = 0LL;
    int64_t v317 = 0LL;
    int64_t v318 = 0LL;
    int64_t v319 = 0LL;
    int64_t v320 = 0LL;
    int64_t v321 = 0LL;
    int64_t v322 = 0LL;
    int64_t v323 = 0LL;
    int64_t v324 = 0LL;
    int64_t v325 = 0LL;
    int64_t v326 = 0LL;
    int64_t v327 = 0LL;
    int64_t v328 = 0LL;
    int64_t v329 = 0LL;
    int64_t v330 = 0LL;
    int64_t v331 = 0LL;
    int64_t v332 = 0LL;
    int64_t v333 = 0LL;
    int64_t v334 = 0LL;
    int64_t v335 = 0LL;
    int64_t v336 = 0LL;
    int64_t v337 = 0LL;
    int64_t v338 = 0LL;
    int64_t v339 = 0LL;
    int64_t v340 = 0LL;
    int64_t v341 = 0LL;
    int64_t v342 = 0LL;
    int64_t v343 = 0LL;
    int64_t v344 = 0LL;
    int64_t v345 = 0LL;
    int64_t v346 = 0LL;
    int64_t v347 = 0LL;
    int64_t v348 = 0LL;
    int64_t v349 = 0LL;
    int64_t v350 = 0LL;
    int64_t v351 = 0LL;
    int64_t v352 = 0LL;
    int64_t v353 = 0LL;
    int64_t v354 = 0LL;
    int64_t status = 0LL;
    int64_t v356 = 0LL;
    int64_t v357 = 0LL;
    int64_t v358 = 0LL;
    int64_t v359 = 0LL;
    int64_t v360 = 0LL;
    int64_t v361 = 0LL;
    int64_t v362 = 0LL;
    int64_t v363 = 0LL;
    int64_t v364 = 0LL;
    int64_t v365 = 0LL;
    int64_t v366 = 0LL;
    int64_t v367 = 0LL;
    int64_t v368 = 0LL;
    int64_t v369 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 8LL);
    *((int64_t *)(v3)) = v4;
    v5 = (v3 - 8LL);
    *((int64_t *)(v5)) = v6;
    v7 = (v5 - 8LL);
    *((int64_t *)(v7)) = v8;
    v9 = (v7 - 8LL);
    *((int64_t *)(v9)) = v10;
    v11 = (v9 - 8LL);
    *((int64_t *)(v11)) = v12;
    v13 = (v11 - 8LL);
    *((int64_t *)(v13)) = v14;
    v15 = (v13 - 8LL);
    *((int64_t *)(v15)) = v16;
    v17 = (v15 - 88LL);
    v18 = (*((int64_t *)(48LL)));
    v19 = v18;
    v20 = (v19 + 8LL);
    v21 = (*((int64_t *)(v20)));
    v22 = v21;
    v24 = (v23 + 5368726704LL);
    v25 = (*((int64_t *)(v24)));
    v26 = v25;
    v27 = (v23 + 5368746720LL);
    v28 = (*((int64_t *)(v27)));
    v29 = v28;
    while (1) {
        /* phi v31 <- (bb0: v19) (bb3: v39) */
        /* phi v32 <- (bb0: v30) (bb3: v35) */
        v33 = (v31 ^ v31);
        (/* opaque: cmpxchg */ 0);
        /* structurally unreachable: block 4 */
        __builtin_unreachable();
    }
    /* phi v42 <- (bb5: v41) (bb18: v40) */
    v43 = (v23 + 5368726720LL);
    v44 = (*((int64_t *)(v43)));
    v45 = v44;
    v46 = (*((int32_t *)(v45)));
    v47 = v46;
    v48 = (v47 == 1LL);
    if (v48) {
L0:;
        /* phi v363 <- (bb6: v29) (bb79: status) */
        /* phi v364 <- (bb6: v22) (bb79: v356) */
        /* phi v365 <- (bb6: v36) (bb79: v358) */
        /* phi v366 <- (bb6: v37) (bb79: v359) */
        /* phi v367 <- (bb6: v38) (bb79: v360) */
        v368 = 31LL;
        v369 = ((long long (*)(long long, long long, long long, long long, long long, long long))_amsg_exit)(v363, v364, v365, v368, v366, v367);
        /* structurally unreachable: block 81 */
        __builtin_unreachable();
    } else {
        v49 = (*((int32_t *)(v45)));
        v50 = v49;
        v51 = (v50 & v50);
        v52 = (v51 == 0LL);
        if (v52) {
            v130 = 2LL;
            *((int32_t *)(v45)) = 1LL;
            v131 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v29, v22, v36, v130, v37, v38);
            v132 = (v38 ^ v38);
            v133 = 4LL;
            v134 = (v36 ^ v36);
            v135 = v131;
            v136 = ((long long (*)(long long, long long, long long, long long, long long, long long))setvbuf)(v29, v22, v134, v135, v133, v132);
            v137 = (v23 + 5368713232LL);
            v138 = v137;
            v139 = ((long long (*)(long long, long long, long long, long long, long long, long long))_crt_atexit)(v29, v22, v134, v138, v133, v132);
            v140 = v139;
            v141 = (v139 & v139);
            v142 = (v141 != 0LL);
            if (v142) {
                v347 = ((long long (*)(long long, long long, long long, long long, long long, long long))abort)(v29, v140, v134, v138, v133, v132);
                /* phi v348 <- (bb48: n) (bb77: v29) */
                /* phi v349 <- (bb48: v207) (bb77: v140) */
                /* phi v350 <- (bb48: v237) (bb77: v134) */
                /* phi v351 <- (bb48: v210) (bb77: v133) */
                /* phi v352 <- (bb48: v211) (bb77: v132) */
                v353 = 10LL;
                v354 = ((long long (*)(long long, long long, long long, long long, long long, long long))_amsg_exit)(v348, v349, v350, v353, v351, v352);
L1:;
                /* phi status <- (bb14: v54) (bb78: v348) */
                /* phi v356 <- (bb14: v55) (bb78: v349) */
                /* phi v357 <- (bb14: v87) (bb78: v354) */
                /* phi v358 <- (bb14: v86) (bb78: v350) */
                /* phi v359 <- (bb14: v80) (bb78: v351) */
                /* phi v360 <- (bb14: v60) (bb78: v352) */
                v361 = v357;
                v362 = ((long long (*)(long long, long long, long long, long long, long long, long long))exit)(status, v356, v358, v361, v359, v360);
                goto L0;
            } else {
                v143 = ((long long (*)(long long, long long, long long, long long, long long, long long))_pei386_runtime_relocator)(v29, v140, v134, v138, v133, v132);
                v144 = (v23 + 5368717184LL);
                v145 = v144;
                v146 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v29, v140, v134, v145, v133, v132);
                v147 = (v23 + 5368726688LL);
                v148 = (*((int64_t *)(v147)));
                v149 = v148;
                v150 = (v23 + 5368713216LL);
                v151 = v150;
                *((int64_t *)(v149)) = v146;
                v152 = ((long long (*)(long long, long long, long long, long long, long long, long long))_set_invalid_parameter_handler)(v29, v140, v149, v151, v133, v132);
                v153 = ((long long (*)(long long, long long, long long, long long, long long, long long))_fpreset)(v29, v140, v149, v151, v133, v132);
                v154 = (v23 + 5368726640LL);
                v155 = (*((int64_t *)(v154)));
                v156 = v155;
                *((int32_t *)(v156)) = 1LL;
                v157 = (v23 + 5368726656LL);
                v158 = (*((int64_t *)(v157)));
                v159 = v158;
                *((int32_t *)(v159)) = 1LL;
                v160 = (v23 + 5368726672LL);
                v161 = (*((int64_t *)(v160)));
                v162 = v161;
                *((int32_t *)(v162)) = 1LL;
                v163 = (v23 + 5368726528LL);
                v164 = (*((int64_t *)(v163)));
                v165 = v164;
                v166 = (*((int16_t *)(v165)));
                v167 = (v166 != 23117LL);
                if (v167) {
                } else {
                    v168 = (v165 + 60LL);
                    v169 = (*((int32_t *)(v168)));
                    v170 = v169;
                    v171 = (v165 + v170);
                    v172 = (*((int32_t *)(v171)));
                    v173 = (v172 != 17744LL);
                    if (v173) {
                    } else {
                        v174 = (v171 + 24LL);
                        v175 = (*((int16_t *)(v174)));
                        v176 = v175;
                        v177 = (v176 == 267LL);
                        if (v177) {
                            v186 = (v171 + 116LL);
                            v187 = (*((int32_t *)(v186)));
                            v188 = (v187 <= 14LL);
                            if (v188) {
                            } else {
                                v189 = (v171 + 232LL);
                                v190 = (*((int32_t *)(v189)));
                                v191 = v190;
                                v192 = (v140 ^ v140);
                                (/* opaque: setne */ 0);
                            }
                        } else {
                            v178 = (v176 != 523LL);
                            if (v178) {
                            } else {
                                v179 = (v171 + 132LL);
                                v180 = (*((int32_t *)(v179)));
                                v181 = (v180 <= 14LL);
                                if (v181) {
                                } else {
                                    v182 = (v171 + 248LL);
                                    v183 = (*((int32_t *)(v182)));
                                    v184 = v183;
                                    v185 = (v140 ^ v140);
                                    (/* opaque: setne */ 0);
                                }
                            }
                        }
                    }
                }
                /* phi v193 <- (bb33: v140) (bb34: v140) (bb36: v140) (bb37: v140) (bb38: v185) (bb75: v140) (bb76: v192) */
                /* phi v194 <- (bb33: v149) (bb34: v170) (bb36: v176) (bb37: v176) (bb38: v176) (bb75: v176) (bb76: v176) */
                /* phi v195 <- (bb33: v132) (bb34: v132) (bb36: v132) (bb37: v132) (bb38: v184) (bb75: v132) (bb76: v132) */
                v196 = (v23 + 5368726624LL);
                v197 = (*((int64_t *)(v196)));
                v198 = v197;
                v199 = (v23 + 5368741896LL);
                /* recovered field: base=v23 offset=0x140008008 field=field_140008008 */
                *((int32_t *)(v199)) = v193;
                v200 = (*((int32_t *)(v198)));
                v201 = v200;
                v202 = (v201 & v201);
                v203 = (v202 != 0LL);
                if (v203) {
                    /* phi v340 <- (bb39: v29) (bb63: v333) */
                    /* phi v341 <- (bb39: v193) (bb63: v334) */
                    /* phi v342 <- (bb39: v194) (bb63: v335) */
                    /* phi v343 <- (bb39: v201) (bb63: v336) */
                    /* phi v344 <- (bb39: v195) (bb63: v337) */
                    v345 = 2LL;
                    v346 = ((long long (*)(long long, long long, long long, long long, long long, long long))__set_app_type)(v340, v341, v342, v345, v343, v344);
                } else {
                    v204 = 1LL;
                    v205 = ((long long (*)(long long, long long, long long, long long, long long, long long))__set_app_type)(v29, v193, v194, v204, v201, v195);
                }
                /* phi n <- (bb40: v29) (bb65: v340) */
                /* phi v207 <- (bb40: v193) (bb65: v341) */
                /* phi v208 <- (bb40: v204) (bb65: v345) */
                /* phi v209 <- (bb40: v194) (bb65: v342) */
                /* phi v210 <- (bb40: v201) (bb65: v343) */
                /* phi v211 <- (bb40: v195) (bb65: v344) */
                v212 = ((long long (*)(long long, long long, long long, long long, long long, long long))__p__fmode)(n, v207, v209, v208, v210, v211);
                v213 = (v23 + 5368726832LL);
                v214 = (*((int64_t *)(v213)));
                v215 = v214;
                v216 = (*((int32_t *)(v215)));
                v217 = v216;
                *((int32_t *)(v212)) = v217;
                v218 = ((long long (*)(long long, long long, long long, long long, long long, long long))__p__commode)(n, v207, v217, v208, v210, v211);
                v219 = (v23 + 5368726800LL);
                v220 = (*((int64_t *)(v219)));
                v221 = v220;
                v222 = (*((int32_t *)(v221)));
                v223 = v222;
                *((int32_t *)(v218)) = v223;
                v224 = ((long long (*)(long long, long long, long long, long long, long long, long long))_setargv)(n, v207, v223, v208, v210, v211);
                /* structurally unreachable: block 44 */
                __builtin_unreachable();
            }
        } else {
            v53 = (v23 + 5368741892LL);
            /* recovered field: base=v23 offset=0x140008004 field=field_140008004 */
            *((int32_t *)(v53)) = 1LL;
            /* phi v54 <- (bb8: v29) (bb70: v315) */
            /* phi v55 <- (bb8: v22) (bb70: v316) */
            /* phi v56 <- (bb8: v50) (bb70: v327) */
            /* phi v57 <- (bb8: v32) (bb70: v324) */
            /* phi v58 <- (bb8: v36) (bb70: v321) */
            /* phi v59 <- (bb8: v37) (bb70: v318) */
            /* phi v60 <- (bb8: v38) (bb70: v260) */
            v61 = (v42 & v42);
            v62 = (v61 == 0LL);
            if (v62) {
                v129 = (v56 ^ v56);
                (/* opaque: xchg */ 0);
            }
            v63 = (v23 + 5368726576LL);
            v64 = (*((int64_t *)(v63)));
            v65 = v64;
            v66 = (*((int64_t *)(v65)));
            v67 = v66;
            v68 = (v67 & v67);
            v69 = (v68 == 0LL);
            if (v69) {
            } else {
                v70 = (v59 ^ v59);
                v71 = 2LL;
                v72 = (v57 ^ v57);
                v73 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v54, v55, v71, v72, v70, v60);
            }
            /* phi v74 <- (bb10: v57) (bb11: v72) */
            /* phi v75 <- (bb10: v58) (bb11: v71) */
            /* phi v76 <- (bb10: v59) (bb11: v70) */
            v77 = ((long long (*)(long long, long long, long long, long long, long long, long long))__p___initenv)(v54, v55, v75, v74, v76, v60);
            v78 = (v23 + 5368741904LL);
            v79 = (*((int64_t *)(v78)));
            v80 = v79;
            v81 = (v23 + 5368741920LL);
            v82 = (*((int32_t *)(v81)));
            v83 = v82;
            *((int64_t *)(v77)) = v80;
            v84 = (v23 + 5368741912LL);
            v85 = (*((int64_t *)(v84)));
            v86 = v85;
            v87 = ((long long (*)(long long, long long, long long, long long, long long, long long))main)(v54, v55, v86, v83, v80, v60);
            v88 = (v23 + 5368741896LL);
            v89 = (*((int32_t *)(v88)));
            v90 = v89;
            v91 = (v90 & v90);
            v92 = (v91 == 0LL);
            if (v92) {
                goto L1;
            } else {
                v93 = (v23 + 5368741892LL);
                v94 = (*((int32_t *)(v93)));
                v95 = v94;
                v96 = (v95 & v95);
                v97 = (v96 == 0LL);
                if (v97) {
                    v124 = (v17 + 60LL);
                    /* recovered field: base=v17 offset=0x3c field=field_3c */
                    *((int32_t *)(v124)) = v87;
                    v125 = ((long long (*)(long long, long long, long long, long long, long long, long long))_cexit)(v54, v55, v95, v90, v80, v60);
                    v126 = (v17 + 60LL);
                    v127 = (*((int32_t *)(v126)));
                    v128 = v127;
                }
                /* phi v98 <- (bb15: v87) (bb21: v128) */
                v99 = (v17 + 88LL);
                v100 = (*((int64_t *)(v99)));
                v101 = (v99 + 8LL);
                v102 = v100;
                v103 = (*((int64_t *)(v101)));
                v104 = (v101 + 8LL);
                v105 = v103;
                v106 = (*((int64_t *)(v104)));
                v107 = (v104 + 8LL);
                v108 = v106;
                v109 = (*((int64_t *)(v107)));
                v110 = (v107 + 8LL);
                v111 = v109;
                v112 = (*((int64_t *)(v110)));
                v113 = (v110 + 8LL);
                v114 = v112;
                v115 = (*((int64_t *)(v113)));
                v116 = (v113 + 8LL);
                v117 = v115;
                v118 = (*((int64_t *)(v116)));
                v119 = (v116 + 8LL);
                v120 = v118;
                v121 = (*((int64_t *)(v119)));
                v122 = (v119 + 8LL);
                v123 = v121;
                return v98;
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
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = arg0;
    int64_t v7 = arg1;
    int64_t v8 = arg2;
    int64_t v9 = arg3;
    int64_t v10 = arg4;
    int64_t v11 = arg5;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;

    v1 = (v0 - 40LL);
    v3 = (v2 + 5368726624LL);
    v4 = (*((int64_t *)(v3)));
    v5 = v4;
    *((int32_t *)(v5)) = 1LL;
    v12 = ((long long (*)(long long, long long, long long, long long, long long, long long))__tmainCRTStartup)(v6, v7, v8, v9, v10, v11);
    v13 = (v1 + 40LL);
    return v12;
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
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = arg0;
    int64_t v7 = arg1;
    int64_t v8 = arg2;
    int64_t v9 = arg3;
    int64_t v10 = arg4;
    int64_t v11 = arg5;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;

    v1 = (v0 - 40LL);
    v3 = (v2 + 5368726624LL);
    v4 = (*((int64_t *)(v3)));
    v5 = v4;
    *((int32_t *)(v5)) = 0LL;
    v12 = ((long long (*)(long long, long long, long long, long long, long long, long long))__tmainCRTStartup)(v6, v7, v8, v9, v10, v11);
    v13 = (v1 + 40LL);
    return v12;
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
/* struct_layouts: pointer=1 stack=1 */
/* switch_tables: 0 */
void __gcc_register_frame(int64_t arg0, int64_t arg1, int64_t arg2) {
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
    int64_t v11 = arg0;
    int64_t v12 = arg1;
    int64_t v13 = arg2;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int8_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int8_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    int64_t v47 = 0LL;
    int64_t v48 = 0LL;
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
    int64_t v59 = 0LL;
    int64_t v60 = 0LL;
    int64_t v61 = 0LL;
    int64_t v62 = 0LL;
    int64_t v63 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 8LL);
    *((int64_t *)(v3)) = v4;
    v5 = (v3 - 56LL);
    v6 = (v5 + 48LL);
    v7 = v6;
    v9 = (v8 + 5368725504LL);
    v10 = v9;
    v16 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v11, v12, v13, v10, v14, v15);
    v17 = v16;
    v18 = (v16 & v16);
    v19 = (v18 == 0LL);
    if (v19) {
        v59 = (v8 + 5368714368LL);
        v60 = v59;
        v61 = (v8 + 5368714352LL);
        v62 = v61;
        v63 = (v8 + 5368721408LL);
        /* recovered field: base=v8 offset=0x140003000 field=field_140003000 */
        *((int64_t *)(v63)) = v60;
L0:;
        /* phi v43 <- (bb5: v39) (bb8: v62) */
        /* phi v44 <- (bb5: v25) (bb8: v15) */
        v45 = (v8 + 5368741984LL);
        v46 = v45;
        v47 = (v8 + 5368729600LL);
        v48 = v47;
        v49 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v11, v12, v46, v48, v43, v44);
    } else {
        v20 = (v8 + 5368725504LL);
        v21 = v20;
        v22 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v11, v12, v13, v21, v14, v15);
        v23 = (v8 + 5368746672LL);
        v24 = (*((int64_t *)(v23)));
        v25 = v24;
        v26 = (v8 + 5368725523LL);
        v27 = v26;
        v28 = v17;
        v29 = (v8 + 5368741952LL);
        /* recovered field: base=v8 offset=0x140008040 field=field_140008040 */
        *((int64_t *)(v29)) = v22;
        v30 = (v7 + -16LL);
        *((int64_t *)(v30)) = v25;
        v31 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v11, v12, v27, v28, v14, v25);
        v32 = (v8 + 5368725545LL);
        v33 = v32;
        v34 = v17;
        v35 = (v7 + -8LL);
        *((int64_t *)(v35)) = v31;
        v36 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v11, v12, v33, v34, v14, v25);
        v37 = (v7 + -8LL);
        v38 = (*((int64_t *)(v37)));
        v39 = v38;
        v40 = (v8 + 5368721408LL);
        /* recovered field: base=v8 offset=0x140003000 field=field_140003000 */
        *((int64_t *)(v40)) = v36;
        v41 = (v39 & v39);
        v42 = (v41 == 0LL);
        if (v42) {
        } else {
            goto L0;
        }
    }
    v50 = (v8 + 5368714560LL);
    v51 = v50;
    v52 = (v5 + 56LL);
    v53 = (*((int64_t *)(v52)));
    v54 = (v52 + 8LL);
    v55 = v53;
    v56 = (*((int64_t *)(v54)));
    v57 = (v54 + 8LL);
    v58 = v56;
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
/* struct_layouts: pointer=1 stack=0 */
/* switch_tables: 0 */
int64_t __gcc_deregister_frame(int64_t arg0, int64_t arg1, int64_t arg2) {
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
    int8_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = arg0;
    int64_t v14 = arg1;
    int64_t v15 = arg2;
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
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = v1;
    v4 = (v1 - 32LL);
    v6 = (v5 + 5368721408LL);
    v7 = (*((int64_t *)(v6)));
    v8 = v7;
    v9 = (v8 & v8);
    v10 = (v9 == 0LL);
    if (v10) {
    } else {
        v11 = (v5 + 5368729600LL);
        v12 = v11;
        v18 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v13, v14, v15, v12, v16, v17);
    }
    /* phi v19 <- (bb0: v8) (bb1: v18) */
    v20 = (v5 + 5368741952LL);
    v21 = (*((int64_t *)(v20)));
    v22 = v21;
    v23 = (v22 & v22);
    v24 = (v23 == 0LL);
    if (v24) {
        v29 = (v4 + 32LL);
        v30 = (*((int64_t *)(v29)));
        v31 = (v29 + 8LL);
        v32 = v30;
        return v19;
    } else {
        v25 = (v4 + 32LL);
        v26 = (*((int64_t *)(v25)));
        v27 = (v25 + 8LL);
        v28 = v26;
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
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int8_t v9 = 0LL;
    int64_t v10 = arg2;
    int64_t v11 = 0LL;
    int64_t v12 = arg0;
    int64_t v13 = arg1;
    int64_t v14 = arg3;
    int64_t v15 = arg4;
    int64_t v16 = arg5;
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
    int8_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;

    v1 = (v0 - 40LL);
    v3 = (v2 + 5368721424LL);
    v4 = (*((int64_t *)(v3)));
    v5 = v4;
    v6 = (*((int64_t *)(v5)));
    v7 = v6;
    v8 = (v7 & v7);
    v9 = (v8 == 0LL);
    if (v9) {
    } else {
        while (1) {
            /* phi v11 <- (bb1: v10) (bb3: v22) */
            v17 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v12, v13, v11, v14, v15, v16);
            v18 = (v2 + 5368721424LL);
            v19 = (*((int64_t *)(v18)));
            v20 = v19;
            v21 = (v20 + 8LL);
            v22 = v21;
            v24 = (*((int64_t *)(v21)));
            v25 = v24;
            *((int64_t *)(v18)) = v22;
            v27 = (v25 & v25);
            v28 = (v27 != 0LL);
            if (v28) {
                continue;
            }
        }
    }
    /* phi v29 <- (bb0: v7) (bb3: v25) */
    v30 = (v1 + 40LL);
    return v29;
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
    int8_t v13 = 0LL;
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
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = arg0;
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
    int64_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    int64_t v47 = 0LL;
    int64_t v48 = 0LL;
    int64_t v49 = 0LL;
    int64_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int8_t v53 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 8LL);
    *((int64_t *)(v3)) = v4;
    v5 = (v3 - 40LL);
    v7 = (v6 + 5368726512LL);
    v8 = (*((int64_t *)(v7)));
    v9 = v8;
    v10 = (*((int64_t *)(v9)));
    v11 = v10;
    v12 = v11;
    v13 = (v11 == -1LL);
    if (v13) {
        v44 = (v11 ^ v11);
        while (1) {
            /* phi v45 <- (bb7: v44) (bb8: v49) */
            v46 = (v45 + 1LL);
            v47 = v46;
            v48 = v45;
            v49 = v47;
            v50 = (v47 * 8LL);
            v51 = (v9 + v50);
            v52 = (*((int64_t *)(v51)));
            v53 = (v52 != 0LL);
            if (v53) {
                continue;
            } else {
                break;
            }
        }
    }
    /* phi v15 <- (bb0: v12) (bb9: v48) */
    /* phi v16 <- (bb0: v14) (bb9: v47) */
    v17 = (v15 & v15);
    v18 = (v17 == 0LL);
    if (v18) {
    } else {
        v19 = v15;
        v20 = (v15 - 1LL);
        v21 = (v19 * 8LL);
        v22 = (v9 + v21);
        v23 = v22;
        v24 = (v19 - v20);
        v25 = (v9 + -8LL);
        v26 = (v24 * 8LL);
        v27 = (v25 + v26);
        v28 = v27;
        while (1) {
            /* phi v29 <- (bb2: v23) (bb4: v33) */
            v32 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v30, v28, v9, v20, v16, v31);
            v33 = (v29 - 8LL);
            v34 = (v33 != v28);
            if (v34) {
                continue;
            }
        }
    }
    v35 = (v6 + 5368714624LL);
    v36 = v35;
    v37 = (v5 + 40LL);
    v38 = (*((int64_t *)(v37)));
    v39 = (v37 + 8LL);
    v40 = v38;
    v41 = (*((int64_t *)(v39)));
    v42 = (v39 + 8LL);
    v43 = v41;
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
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int32_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;

    v1 = (v0 + 5368742048LL);
    v2 = (*((int32_t *)(v1)));
    v3 = v2;
    v4 = (v3 & v3);
    v5 = (v4 == 0LL);
    if (v5) {
        v6 = (v0 + 5368742048LL);
        *((int32_t *)(v6)) = 1LL;
        /* structurally unreachable: block 3 */
        __builtin_unreachable();
    } else {
        return v3;
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
    int64_t v1 = 0LL;
    int64_t v2 = arg1;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int32_t v10 = 0LL;
    int8_t v11 = 0LL;
    int64_t v12 = arg2;
    int8_t v13 = 0LL;
    int8_t v14 = 0LL;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
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
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int8_t v33 = 0LL;
    int64_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int8_t v38 = 0LL;
    int64_t v39 = arg0;
    int64_t v40 = arg3;
    int64_t v41 = arg4;
    int64_t v42 = arg5;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int8_t v46 = 0LL;
    int64_t v47 = 0LL;
    int64_t v48 = 0LL;
    int64_t v49 = 0LL;
    int64_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int64_t v53 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 8LL);
    *((int64_t *)(v3)) = v4;
    v5 = (v3 - 40LL);
    v7 = (v6 + 5368726480LL);
    v8 = (*((int64_t *)(v7)));
    v9 = v8;
    v10 = (*((int32_t *)(v9)));
    v11 = (v10 == 2LL);
    if (v11) {
    } else {
        *((int32_t *)(v9)) = 2LL;
    }
    v13 = (v12 == 2LL);
    if (v13) {
        v29 = (v6 + 5368726976LL);
        v30 = v29;
        v33 = (v30 == v30);
        if (v33) {
L0:;
            v22 = (v5 + 40LL);
            v23 = (*((int64_t *)(v22)));
            v24 = (v22 + 8LL);
            v25 = v23;
            v26 = (*((int64_t *)(v24)));
            v27 = (v24 + 8LL);
            v28 = v26;
            return v9;
        } else {
            while (1) {
                /* phi v34 <- (bb7: v30) (bb10: v45) */
                v35 = (*((int64_t *)(v34)));
                v36 = v35;
                v37 = (v36 & v36);
                v38 = (v37 == 0LL);
                if (v38) {
                } else {
                    v43 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v39, v30, v12, v40, v41, v42);
                }
                /* phi v44 <- (bb8: v36) (bb9: v43) */
                v45 = (v34 + 8LL);
                v46 = (v45 != v30);
                if (v46) {
                    continue;
                } else {
                    break;
                }
            }
            v47 = (v5 + 40LL);
            v48 = (*((int64_t *)(v47)));
            v49 = (v47 + 8LL);
            v50 = v48;
            v51 = (*((int64_t *)(v49)));
            v52 = (v49 + 8LL);
            v53 = v51;
            return v44;
        }
    } else {
        v14 = (v12 == 1LL);
        if (v14) {
            v15 = (v5 + 40LL);
            v16 = (*((int64_t *)(v15)));
            v17 = (v15 + 8LL);
            v18 = v16;
            v19 = (*((int64_t *)(v17)));
            v20 = (v17 + 8LL);
            v21 = v19;
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
int64_t _matherr(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = arg1;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = arg3;
    int32_t v7 = 0LL;
    int8_t v8 = 0LL;
    int32_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    int32_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = arg0;
    int64_t v26 = arg2;
    int64_t v27 = arg4;
    int64_t v28 = arg5;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 8LL);
    *((int64_t *)(v3)) = v4;
    v5 = (v3 - 120LL);
    (/* opaque: movaps */ 0);
    (/* opaque: movaps */ 0);
    (/* opaque: movaps */ 0);
    v7 = (*((int32_t *)(v6)));
    v8 = (v7 > 6LL);
    if (v8) {
        v19 = (v11 + 5368725926LL);
        v20 = v19;
        (/* opaque: movsd */ 0);
        (/* opaque: movsd */ 0);
        (/* opaque: movsd */ 0);
        v21 = (v6 + 8LL);
        v22 = (*((int64_t *)(v21)));
        v23 = v22;
        v24 = 2LL;
        v29 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v25, v23, v26, v24, v27, v28);
        (/* opaque: movsd */ 0);
        v30 = v20;
        v31 = (v11 + 5368725944LL);
        v32 = v31;
        (/* opaque: movsd */ 0);
        v33 = v23;
        v34 = v29;
        (/* opaque: movsd */ 0);
        v35 = ((long long (*)(long long, long long, long long, long long, long long, long long))fprintf)(v25, v23, v32, v34, v30, v33);
        (/* opaque: movaps */ 0);
        (/* opaque: movaps */ 0);
        v36 = (v35 ^ v35);
        (/* opaque: movaps */ 0);
        v37 = (v5 + 120LL);
        v38 = (*((int64_t *)(v37)));
        v39 = (v37 + 8LL);
        v40 = v38;
        v41 = (*((int64_t *)(v39)));
        v42 = (v39 + 8LL);
        v43 = v41;
        return v36;
    } else {
        v9 = (*((int32_t *)(v6)));
        v10 = v9;
        v12 = (v11 + 5368725988LL);
        v13 = v12;
        v14 = (v10 * 4LL);
        v15 = (v13 + v14);
        v16 = (*((int32_t *)(v15)));
        v17 = v16;
        v18 = (v17 + v13);
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
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = arg0;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = arg2;
    int64_t v13 = 0LL;
    int64_t v14 = arg3;
    int64_t v15 = 0LL;
    int64_t v16 = arg1;
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
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 8LL);
    *((int64_t *)(v3)) = v4;
    v5 = (v3 - 56LL);
    v7 = v6;
    v8 = (v5 + 88LL);
    v9 = v8;
    v10 = 2LL;
    v11 = (v5 + 96LL);
    /* recovered field: base=v5 offset=0x60 field=field_60 */
    *((int64_t *)(v11)) = v12;
    v13 = (v5 + 104LL);
    /* recovered field: base=v5 offset=0x68 field=field_68 */
    *((int64_t *)(v13)) = v14;
    /* recovered field: base=v5 offset=0x58 field=field_58 */
    *((int64_t *)(v8)) = v16;
    v17 = (v5 + 40LL);
    *((int64_t *)(v17)) = v9;
    v19 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v18, v2, v16, v10, v12, v14);
    v21 = (v20 + 5368726016LL);
    v22 = v21;
    v23 = v19;
    v24 = ((long long (*)(long long, long long, long long, long long, long long, long long))fprintf)(v18, v2, v22, v23, v12, v14);
    v25 = (v5 + 40LL);
    v26 = (*((int64_t *)(v25)));
    v27 = v26;
    v28 = 2LL;
    v29 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v18, v27, v22, v28, v12, v14);
    v30 = v7;
    v31 = v27;
    v32 = v29;
    v33 = ((long long (*)(long long, long long, long long, long long, long long, long long))vfprintf)(v18, v27, v30, v32, v31, v14);
    v34 = ((long long (*)(long long, long long, long long, long long, long long, long long))abort)(v18, v27, v30, v32, v31, v14);
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
/* struct_layouts: pointer=6 stack=1 */
/* switch_tables: 0 */
int64_t mark_section_writable(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = arg0;
    int64_t v3 = 0LL;
    int64_t v4 = arg1;
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int32_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = arg3;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int8_t v15 = 0LL;
    int64_t v16 = arg5;
    int64_t v17 = arg4;
    int64_t v18 = arg2;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int8_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int32_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int8_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int8_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    int64_t v47 = 0LL;
    int64_t v48 = 0LL;
    int64_t v49 = 0LL;
    int64_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int8_t v53 = 0LL;
    int64_t v54 = 0LL;
    int64_t v55 = 0LL;
    int64_t v56 = 0LL;
    int64_t v57 = 0LL;
    int64_t v58 = 0LL;
    int64_t v59 = 0LL;
    int64_t v60 = 0LL;
    int64_t v61 = 0LL;
    int64_t v62 = 0LL;
    int64_t v63 = 0LL;
    int64_t v64 = 0LL;
    int32_t v65 = 0LL;
    int64_t v66 = 0LL;
    int64_t v67 = 0LL;
    int64_t v68 = 0LL;
    int64_t v69 = 0LL;
    int64_t v70 = 0LL;
    int64_t v71 = 0LL;
    int64_t v72 = 0LL;
    int64_t v73 = 0LL;
    int64_t v74 = 0LL;
    int64_t v75 = 0LL;
    int64_t v76 = 0LL;
    int64_t v77 = 0LL;
    int64_t v78 = 0LL;
    int8_t v79 = 0LL;
    int64_t v80 = 0LL;
    int32_t v81 = 0LL;
    int64_t v82 = 0LL;
    int64_t v83 = 0LL;
    int64_t v84 = 0LL;
    int64_t v85 = 0LL;
    int64_t v86 = 0LL;
    int64_t v87 = 0LL;
    int64_t v88 = 0LL;
    int64_t v89 = 0LL;
    int64_t v90 = 0LL;
    int64_t v91 = 0LL;
    int64_t v92 = 0LL;
    int64_t v93 = 0LL;
    int64_t v94 = 0LL;
    int64_t v95 = 0LL;
    int64_t v96 = 0LL;
    int64_t v97 = 0LL;
    int64_t v98 = 0LL;
    int64_t v99 = 0LL;
    int64_t v100 = 0LL;
    int64_t v101 = 0LL;
    int64_t v102 = 0LL;
    int64_t v103 = 0LL;
    int64_t v104 = 0LL;
    int8_t v105 = 0LL;
    int64_t v106 = 0LL;
    int64_t v107 = 0LL;
    int64_t v108 = 0LL;
    int64_t v109 = 0LL;
    int64_t v110 = 0LL;
    int64_t v111 = 0LL;
    int64_t v112 = 0LL;
    int32_t v113 = 0LL;
    int64_t v114 = 0LL;
    int64_t v115 = 0LL;
    int64_t v116 = 0LL;
    int64_t v117 = 0LL;
    int64_t v118 = 0LL;
    int64_t v119 = 0LL;
    int32_t v120 = 0LL;
    int64_t v121 = 0LL;
    int64_t v122 = 0LL;
    int64_t v123 = 0LL;
    int64_t v124 = 0LL;
    int64_t v125 = 0LL;
    int64_t v126 = 0LL;
    int64_t v127 = 0LL;
    int64_t v128 = 0LL;
    int64_t v129 = 0LL;
    int64_t v130 = 0LL;
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
    int64_t v146 = 0LL;
    int64_t v147 = 0LL;
    int64_t v148 = 0LL;
    int64_t v149 = 0LL;
    int64_t v150 = 0LL;
    int64_t v151 = 0LL;
    int64_t v152 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 8LL);
    *((int64_t *)(v3)) = v4;
    v5 = (v3 - 8LL);
    *((int64_t *)(v5)) = v6;
    v7 = (v5 - 80LL);
    v9 = (v8 + 5368742148LL);
    v10 = (*((int32_t *)(v9)));
    v11 = v10;
    v13 = v12;
    v14 = (v11 & v11);
    v15 = (v14 <= 0LL);
    if (v15) {
        /* phi v146 <- (bb0: v2) (bb19: v51) */
        /* phi v147 <- (bb0: v11) (bb19: v44) */
        /* phi v148 <- (bb0: v13) (bb19: v99) */
        /* phi v149 <- (bb0: v16) (bb19: v101) */
        /* phi v150 <- (bb0: v17) (bb19: v95) */
        /* phi v151 <- (bb0: v18) (bb19: v109) */
        v152 = (v147 ^ v147);
L2:;
        /* phi v43 <- (bb4: v2) (bb20: v146) */
        /* phi v44 <- (bb4: v11) (bb20: v152) */
        /* phi v45 <- (bb4: v13) (bb20: v148) */
        /* phi v46 <- (bb4: v40) (bb20: v149) */
        /* phi v47 <- (bb4: v38) (bb20: v150) */
        /* phi v48 <- (bb4: v39) (bb20: v151) */
        v49 = v45;
        v50 = ((long long (*)(long long, long long, long long, long long, long long, long long))__mingw_GetSectionForAddress)(v43, v44, v48, v49, v47, v46);
        v51 = v50;
        v52 = (v50 & v50);
        v53 = (v52 == 0LL);
        if (v53) {
L0:;
            /* phi v129 <- (bb6: v45) (bb21: v60) */
            /* phi v130 <- (bb6: v47) (bb21: v127) */
            v131 = v129;
            v132 = (v8 + 5368726048LL);
            v133 = v132;
            v134 = ((long long (*)(long long, long long, long long, long long, long long, long long))__report_error)(v51, v44, v131, v133, v130, v46);
            /* structurally unreachable: block 23 */
            __builtin_unreachable();
        } else {
            v54 = (v8 + 5368742152LL);
            v55 = (*((int64_t *)(v54)));
            v56 = v55;
            v57 = (v44 * 4LL);
            v58 = (v44 + v57);
            v59 = v58;
            v60 = (v59 << 3LL);
            v61 = (v56 + v60);
            v62 = (v61 + 32LL);
            /* recovered field: base=v61 offset=0x20 field=field_20 */
            *((int64_t *)(v62)) = v51;
            *((int32_t *)(v61)) = 0LL;
            v63 = ((long long (*)(long long, long long, long long, long long, long long, long long))_GetPEImageBase)(v51, v44, v48, v49, v47, v46);
            v64 = (v51 + 12LL);
            v65 = (*((int32_t *)(v64)));
            v66 = v65;
            v67 = 48LL;
            v68 = (v63 + v66);
            v69 = v68;
            v70 = (v8 + 5368742152LL);
            v71 = (*((int64_t *)(v70)));
            v72 = v71;
            v73 = (v7 + 32LL);
            v74 = v73;
            v75 = (v72 + 24LL);
            v76 = (v75 + v60);
            *((int64_t *)(v76)) = v69;
            v77 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v51, v44, v74, v69, v67, v46);
            v78 = (v77 & v77);
            v79 = (v78 == 0LL);
            if (v79) {
                v116 = (v8 + 5368742152LL);
                v117 = (*((int64_t *)(v116)));
                v118 = v117;
                v119 = (v51 + 8LL);
                v120 = (*((int32_t *)(v119)));
                v121 = v120;
                v122 = (v8 + 5368726080LL);
                v123 = v122;
                v124 = (v118 + 24LL);
                v125 = (v124 + v60);
                v126 = (*((int64_t *)(v125)));
                v127 = v126;
                v128 = ((long long (*)(long long, long long, long long, long long, long long, long long))__report_error)(v51, v44, v121, v123, v127, v46);
                goto L0;
            } else {
                v80 = (v7 + 68LL);
                v81 = (*((int32_t *)(v80)));
                v82 = v81;
                v83 = (v82 + -4LL);
                v84 = v83;
                v85 = (v84 & -5LL);
                /* structurally unreachable: block 10 */
                __builtin_unreachable();
            }
        }
    } else {
        v19 = (v8 + 5368742152LL);
        v20 = (*((int64_t *)(v19)));
        v21 = v20;
        v22 = (v16 ^ v16);
        v23 = (v21 + 24LL);
        while (1) {
            /* phi v24 <- (bb1: v23) (bb4: v41) */
            /* phi v25 <- (bb1: v22) (bb4: v40) */
            /* phi v26 <- (bb1: v18) (bb4: v39) */
            v27 = (*((int64_t *)(v24)));
            v28 = v27;
            v29 = (v13 < v28);
            if (v29) {
L1:;
                /* phi v38 <- (bb2: v28) (bb3: v36) */
                /* phi v39 <- (bb2: v26) (bb3: v35) */
                v40 = (v25 + 1LL);
                v41 = (v24 + 40LL);
                v42 = (v40 != v11);
                if (v42) {
                    continue;
                } else {
                    break;
                }
            } else {
                v30 = (v24 + 8LL);
                v31 = (*((int64_t *)(v30)));
                v32 = v31;
                v33 = (v32 + 8LL);
                v34 = (*((int32_t *)(v33)));
                v35 = v34;
                v36 = (v28 + v35);
                v37 = (v13 < v36);
                if (v37) {
                    /* phi v135 <- (bb3: v24) (bb12: v111) */
                    v136 = (v7 + 80LL);
                    v137 = (*((int64_t *)(v136)));
                    v138 = (v136 + 8LL);
                    v139 = v137;
                    v140 = (*((int64_t *)(v138)));
                    v141 = (v138 + 8LL);
                    v142 = v140;
                    v143 = (*((int64_t *)(v141)));
                    v144 = (v141 + 8LL);
                    v145 = v143;
                    return v135;
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
/* struct_layouts: pointer=7 stack=1 */
/* switch_tables: 0 */
int64_t _pei386_runtime_relocator(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
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
    int64_t v12 = arg0;
    int64_t v13 = 0LL;
    int64_t v14 = arg1;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int32_t v22 = 0LL;
    int64_t src = 0LL;
    int64_t v24 = 0LL;
    int8_t v25 = 0LL;
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
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
    int64_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int64_t v53 = 0LL;
    int64_t v54 = 0LL;
    int64_t v55 = arg2;
    int64_t v56 = arg3;
    int64_t v57 = arg4;
    int64_t v58 = arg5;
    int64_t v59 = 0LL;
    int64_t v60 = 0LL;
    int64_t v61 = 0LL;
    int64_t v62 = 0LL;
    int64_t v63 = 0LL;
    int64_t v64 = 0LL;
    int64_t v65 = 0LL;
    int64_t v66 = 0LL;
    int64_t v67 = 0LL;
    int64_t v68 = 0LL;
    int64_t v69 = 0LL;
    int64_t dst = 0LL;
    int64_t v71 = 0LL;
    int64_t v72 = 0LL;
    int64_t v73 = 0LL;
    int64_t v74 = 0LL;
    int64_t v75 = 0LL;
    int64_t v76 = 0LL;
    int64_t v77 = 0LL;
    int64_t v78 = 0LL;
    int64_t v79 = 0LL;
    int64_t v80 = 0LL;
    int8_t v81 = 0LL;
    int8_t v82 = 0LL;
    int64_t v83 = 0LL;
    int64_t v84 = 0LL;
    int64_t v85 = 0LL;
    int32_t v86 = 0LL;
    int64_t v87 = 0LL;
    int64_t v88 = 0LL;
    int8_t v89 = 0LL;
    int64_t v90 = 0LL;
    int32_t v91 = 0LL;
    int64_t v92 = 0LL;
    int64_t v93 = 0LL;
    int8_t v94 = 0LL;
    int64_t v95 = 0LL;
    int64_t v96 = 0LL;
    int64_t v97 = 0LL;
    int64_t v98 = 0LL;
    int64_t v99 = 0LL;
    int32_t v100 = 0LL;
    int64_t v101 = 0LL;
    int8_t v102 = 0LL;
    int64_t v103 = 0LL;
    int64_t v104 = 0LL;
    int64_t v105 = 0LL;
    int64_t v106 = 0LL;
    int64_t v107 = 0LL;
    int64_t v108 = 0LL;
    int8_t v109 = 0LL;
    int64_t v110 = 0LL;
    int32_t v111 = 0LL;
    int64_t v112 = 0LL;
    int64_t v113 = 0LL;
    int32_t v114 = 0LL;
    int64_t v115 = 0LL;
    int64_t v116 = 0LL;
    int32_t v117 = 0LL;
    int64_t v118 = 0LL;
    int64_t v119 = 0LL;
    int64_t v120 = 0LL;
    int64_t v121 = 0LL;
    int64_t v122 = 0LL;
    int64_t v123 = 0LL;
    int8_t v124 = 0LL;
    int64_t v125 = 0LL;
    int64_t v126 = 0LL;
    int8_t v127 = 0LL;
    int8_t v128 = 0LL;
    int8_t v129 = 0LL;
    int16_t v130 = 0LL;
    int64_t v131 = 0LL;
    int64_t v132 = 0LL;
    int64_t v133 = 0LL;
    int64_t v134 = 0LL;
    int64_t v135 = 0LL;
    int64_t v136 = 0LL;
    int64_t v137 = 0LL;
    int8_t v138 = 0LL;
    int8_t v139 = 0LL;
    int64_t v140 = 0LL;
    int64_t v141 = 0LL;
    int64_t v142 = 0LL;
    int64_t v143 = 0LL;
    int64_t n = 0LL;
    int64_t v145 = 0LL;
    int64_t v146 = 0LL;
    int8_t v147 = 0LL;
    int64_t v148 = 0LL;
    int64_t v149 = 0LL;
    int64_t v150 = 0LL;
    int64_t v151 = 0LL;
    int64_t v152 = 0LL;
    int64_t v153 = 0LL;
    int64_t v154 = 0LL;
    int8_t v155 = 0LL;
    int8_t v156 = 0LL;
    int64_t v157 = 0LL;
    int64_t v158 = 0LL;
    int64_t v159 = 0LL;
    int64_t v160 = 0LL;
    int64_t n_1 = 0LL;
    int64_t v162 = 0LL;
    int64_t v163 = 0LL;
    int8_t v164 = 0LL;
    int64_t v165 = 0LL;
    int64_t v166 = 0LL;
    int64_t v167 = 0LL;
    int64_t v168 = 0LL;
    int64_t v169 = 0LL;
    int64_t v170 = 0LL;
    int64_t v171 = 0LL;
    int64_t v172 = 0LL;
    int64_t v173 = 0LL;
    int64_t v174 = 0LL;
    int64_t v175 = 0LL;
    int64_t n_2 = 0LL;
    int64_t v177 = 0LL;
    int64_t v178 = 0LL;
    int8_t v179 = 0LL;
    int64_t v180 = 0LL;
    int64_t v181 = 0LL;
    int64_t v182 = 0LL;
    int64_t v183 = 0LL;
    int32_t v184 = 0LL;
    int64_t v185 = 0LL;
    int64_t v186 = 0LL;
    int64_t v187 = 0LL;
    int64_t v188 = 0LL;
    int64_t v189 = 0LL;
    int64_t v190 = 0LL;
    int64_t v191 = 0LL;
    int64_t v192 = 0LL;
    int64_t v193 = 0LL;
    int8_t v194 = 0LL;
    int8_t v195 = 0LL;
    int64_t v196 = 0LL;
    int64_t v197 = 0LL;
    int64_t v198 = 0LL;
    int64_t v199 = 0LL;
    int64_t n_3 = 0LL;
    int64_t v201 = 0LL;
    int64_t v202 = 0LL;
    int64_t v203 = 0LL;
    int64_t v204 = 0LL;
    int64_t v205 = 0LL;
    int64_t v206 = 0LL;
    int64_t v207 = 0LL;
    int64_t v208 = 0LL;
    int64_t v209 = 0LL;
    int64_t v210 = 0LL;
    int64_t v211 = 0LL;
    int64_t v212 = 0LL;
    int64_t v213 = 0LL;
    int64_t v214 = 0LL;
    int64_t v215 = 0LL;
    int64_t v216 = 0LL;
    int64_t v217 = 0LL;
    int32_t v218 = 0LL;
    int64_t v219 = 0LL;
    int64_t v220 = 0LL;
    int8_t v221 = 0LL;
    int64_t v222 = 0LL;
    int64_t v223 = 0LL;
    int64_t v224 = 0LL;
    int64_t v225 = 0LL;
    int64_t v226 = 0LL;
    int64_t v227 = 0LL;
    int64_t v228 = 0LL;
    int64_t v229 = 0LL;
    int64_t v230 = 0LL;
    int64_t v231 = 0LL;
    int32_t v232 = 0LL;
    int64_t v233 = 0LL;
    int64_t v234 = 0LL;
    int8_t v235 = 0LL;
    int64_t v236 = 0LL;
    int64_t v237 = 0LL;
    int64_t v238 = 0LL;
    int64_t v239 = 0LL;
    int64_t v240 = 0LL;
    int64_t v241 = 0LL;
    int64_t v242 = 0LL;
    int64_t v243 = 0LL;
    int64_t v244 = 0LL;
    int64_t v245 = 0LL;
    int64_t v246 = 0LL;
    int64_t v247 = 0LL;
    int32_t v248 = 0LL;
    int8_t v249 = 0LL;
    int32_t v250 = 0LL;
    int64_t v251 = 0LL;
    int64_t v252 = 0LL;
    int8_t v253 = 0LL;
    int64_t v254 = 0LL;
    int32_t v255 = 0LL;
    int64_t v256 = 0LL;
    int64_t v257 = 0LL;
    int8_t v258 = 0LL;
    int64_t v259 = 0LL;
    int32_t v260 = 0LL;
    int64_t v261 = 0LL;
    int64_t v262 = 0LL;
    int8_t v263 = 0LL;
    int64_t v264 = 0LL;
    int64_t v265 = 0LL;
    int64_t v266 = 0LL;
    int64_t v267 = 0LL;
    int64_t v268 = 0LL;
    int64_t v269 = 0LL;
    int8_t v270 = 0LL;
    int64_t v271 = 0LL;
    int64_t v272 = 0LL;
    int64_t v273 = 0LL;
    int64_t v274 = 0LL;
    int64_t v275 = 0LL;
    int64_t v276 = 0LL;
    int64_t v277 = 0LL;
    int64_t v278 = 0LL;
    int64_t v279 = 0LL;
    int32_t v280 = 0LL;
    int64_t v281 = 0LL;
    int32_t v282 = 0LL;
    int64_t v283 = 0LL;
    int64_t v284 = 0LL;
    int64_t v285 = 0LL;
    int32_t v286 = 0LL;
    int64_t v287 = 0LL;
    int64_t v288 = 0LL;
    int64_t v289 = 0LL;
    int64_t v290 = 0LL;
    int64_t v291 = 0LL;
    int64_t v292 = 0LL;
    int64_t n_4 = 0LL;
    int64_t v294 = 0LL;
    int64_t v295 = 0LL;
    int64_t v296 = 0LL;
    int8_t v297 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 8LL);
    *((int64_t *)(v3)) = v4;
    v5 = (v3 - 8LL);
    *((int64_t *)(v5)) = v6;
    v7 = (v5 - 8LL);
    *((int64_t *)(v7)) = v8;
    v9 = (v7 - 8LL);
    *((int64_t *)(v9)) = v10;
    v11 = (v9 - 8LL);
    *((int64_t *)(v11)) = v12;
    v13 = (v11 - 8LL);
    *((int64_t *)(v13)) = v14;
    v15 = (v13 - 8LL);
    *((int64_t *)(v15)) = v16;
    v17 = (v15 - 72LL);
    v18 = (v17 + 64LL);
    v19 = v18;
    v21 = (v20 + 5368742144LL);
    v22 = (*((int32_t *)(v21)));
    src = v22;
    v24 = (src & src);
    v25 = (v24 == 0LL);
    if (v25) {
        v54 = (v20 + 5368742144LL);
        /* recovered field: base=v20 offset=0x140008100 field=field_140008100 */
        *((int32_t *)(v54)) = 1LL;
        v59 = ((long long (*)(long long, long long, long long, long long, long long, long long))__mingw_GetSectionCount)(v12, src, v55, v56, v57, v58);
        (/* opaque: cdqe */ 0);
        v60 = (v59 * 4LL);
        v61 = (v59 + v60);
        v62 = v61;
        v63 = (v62 * 8LL);
        v64 = (15LL + v63);
        v65 = v64;
        v66 = (v65 & -16LL);
        v67 = ((long long (*)(long long, long long, long long, long long, long long, long long))fn_1400027e0)(v12, src, v55, v56, v57, v58);
        v68 = (v20 + 5368726544LL);
        v69 = (*((int64_t *)(v68)));
        dst = v69;
        v71 = (v20 + 5368726560LL);
        v72 = (*((int64_t *)(v71)));
        v73 = v72;
        v74 = (v17 - v67);
        v75 = (v20 + 5368742148LL);
        /* recovered field: base=v20 offset=0x140008104 field=field_140008104 */
        *((int32_t *)(v75)) = 0LL;
        v76 = (v74 + 48LL);
        v77 = v76;
        v78 = (v20 + 5368742152LL);
        /* recovered field: base=v20 offset=0x140008108 field=field_140008108 */
        *((int64_t *)(v78)) = v77;
        v79 = dst;
        v80 = (v79 - v73);
        v81 = (v80 <= 7LL);
        if (v81) {
L0:;
            /* phi v27 <- (bb0: v26) (bb4: v80) (bb10: v96) (bb31: v219) (bb36: v244) (bb40: v266) */
            v28 = (v19 + 8LL);
            v29 = v28;
            v30 = (*((int64_t *)(v29)));
            v31 = (v29 + 8LL);
            v32 = v30;
            v33 = (*((int64_t *)(v31)));
            v34 = (v31 + 8LL);
            v35 = v33;
            v36 = (*((int64_t *)(v34)));
            v37 = (v34 + 8LL);
            v38 = v36;
            v39 = (*((int64_t *)(v37)));
            v40 = (v37 + 8LL);
            v41 = v39;
            v42 = (*((int64_t *)(v40)));
            v43 = (v40 + 8LL);
            v44 = v42;
            v45 = (*((int64_t *)(v43)));
            v46 = (v43 + 8LL);
            v47 = v45;
            v48 = (*((int64_t *)(v46)));
            v49 = (v46 + 8LL);
            v50 = v48;
            v51 = (*((int64_t *)(v49)));
            v52 = (v49 + 8LL);
            v53 = v51;
            return v27;
        } else {
            v82 = (v80 > 11LL);
            if (v82) {
                v250 = (*((int32_t *)(v73)));
                v251 = v250;
                v252 = (v251 & v251);
                v253 = (v252 != 0LL);
                if (v253) {
L2:;
                    /* phi v265 <- (bb6: v83) (bb7: v83) (bb38: v73) (bb39: v73) */
                    /* phi v266 <- (bb6: v80) (bb7: v92) (bb38: v80) (bb39: v80) */
                    /* phi v267 <- (bb6: v87) (bb7: v87) (bb38: v55) (bb39: v55) */
                    /* phi v268 <- (bb6: v84) (bb7: v84) (bb38: v57) (bb39: v256) */
                    /* phi v269 <- (bb6: v85) (bb7: v85) (bb38: v251) (bb39: v251) */
                    v270 = (v265 >= dst);
                    if (v270) {
                    } else {
                        v271 = (v20 + 5368726528LL);
                        v272 = (*((int64_t *)(v271)));
                        v273 = v272;
                        v274 = (v19 + -8LL);
                        v275 = v274;
                        while (1) {
                            /* phi v276 <- (bb41: v265) (bb44: v284) */
                            /* phi v277 <- (bb41: v267) (bb44: n_4) */
                            /* phi v278 <- (bb41: v268) (bb44: v292) */
                            v279 = (v276 + 4LL);
                            v280 = (*((int32_t *)(v279)));
                            v281 = v280;
                            v282 = (*((int32_t *)(v276)));
                            v283 = v282;
                            v284 = (v276 + 8LL);
                            v285 = (v273 + v281);
                            v286 = (*((int32_t *)(v285)));
                            v287 = (v283 + v286);
                            v288 = (v281 + v273);
                            v289 = v288;
                            v290 = (v19 + -8LL);
                            *((int32_t *)(v290)) = v287;
                            v291 = ((long long (*)(long long, long long, long long, long long, long long, long long))mark_section_writable)(dst, src, v277, v289, v278, v269);
                            v292 = 4LL;
                            n_4 = v275;
                            v294 = (v281 + v273);
                            v295 = v294;
                            v296 = ((long long (*)(long long, long long, long long, long long, long long, long long))memcpy)(dst, src, n_4, v295, v292, v269);
                            v297 = (v284 < dst);
                            if (v297) {
                                continue;
                            } else {
                                break;
                            }
                        }
L1:;
                        /* phi v215 <- (bb21: v125) (bb30: v173) (bb45: v275) */
                        /* phi v216 <- (bb21: v126) (bb30: v172) (bb45: v284) */
                        v217 = (v20 + 5368742148LL);
                        v218 = (*((int32_t *)(v217)));
                        v219 = v218;
                        v220 = (v219 & v219);
                        v221 = (v220 <= 0LL);
                        if (v221) {
                        } else {
                            v222 = (v20 + 5368746736LL);
                            v223 = (*((int64_t *)(v222)));
                            v224 = v223;
                            v225 = (v216 ^ v216);
                            while (1) {
                                /* phi v226 <- (bb32: src) (bb35: v245) */
                                /* phi v227 <- (bb32: v225) (bb35: v246) */
                                v228 = (v20 + 5368742152LL);
                                v229 = (*((int64_t *)(v228)));
                                v230 = v229;
                                v231 = (v230 + v227);
                                v232 = (*((int32_t *)(v231)));
                                v233 = v232;
                                v234 = (v233 & v233);
                                v235 = (v234 == 0LL);
                                if (v235) {
                                } else {
                                    v236 = (v231 + 16LL);
                                    v237 = (*((int64_t *)(v236)));
                                    v238 = v237;
                                    v239 = (v231 + 8LL);
                                    v240 = (*((int64_t *)(v239)));
                                    v241 = v240;
                                    v242 = v215;
                                    v243 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v224, v226, v238, v241, v233, v242);
                                }
                                /* phi v244 <- (bb33: v231) (bb34: v243) */
                                v245 = (v226 + 1LL);
                                v246 = (v227 + 40LL);
                                v247 = (v20 + 5368742148LL);
                                v248 = (*((int32_t *)(v247)));
                                v249 = (v245 < v248);
                                if (v249) {
                                    continue;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    goto L0;
                } else {
                    v254 = (v73 + 4LL);
                    v255 = (*((int32_t *)(v254)));
                    v256 = v255;
                    v257 = (v256 & v256);
                    v258 = (v257 == 0LL);
                    if (v258) {
                        v259 = (v73 + 8LL);
                        v260 = (*((int32_t *)(v259)));
                        v261 = v260;
                        v262 = (v261 & v261);
                        v263 = (v262 != 0LL);
                        if (v263) {
L3:;
                            /* phi v95 <- (bb7: v83) (bb64: v73) */
                            /* phi v96 <- (bb7: v92) (bb64: v80) */
                            /* phi v97 <- (bb7: v84) (bb64: v256) */
                            /* phi v98 <- (bb7: v85) (bb64: v251) */
                            v99 = (v95 + 8LL);
                            v100 = (*((int32_t *)(v99)));
                            v101 = v100;
                            v102 = (v101 != 1LL);
                            if (v102) {
                                /* phi v209 <- (bb8: v101) (bb67: v120) */
                                /* phi v210 <- (bb8: v97) (bb67: v205) */
                                /* phi v211 <- (bb8: v98) (bb67: v122) */
                                v212 = (v20 + 5368726176LL);
                                v213 = v212;
                                v214 = ((long long (*)(long long, long long, long long, long long, long long, long long))__report_error)(dst, src, v209, v213, v210, v211);
                                /* structurally unreachable: block 69 */
                                __builtin_unreachable();
                            } else {
                                v103 = (v95 + 12LL);
                                v104 = (v20 + 5368726528LL);
                                v105 = (*((int64_t *)(v104)));
                                v106 = v105;
                                v107 = (v19 + -8LL);
                                v108 = v107;
                                v109 = (v103 < dst);
                                if (v109) {
                                    while (1) {
                                        /* phi v110 <- (bb9: v103) (bb21: v126) (bb29: v172) */
                                        v111 = (*((int32_t *)(v110)));
                                        v112 = v111;
                                        v113 = (v110 + 8LL);
                                        v114 = (*((int32_t *)(v113)));
                                        v115 = v114;
                                        v116 = (v110 + 4LL);
                                        v117 = (*((int32_t *)(v116)));
                                        v118 = v117;
                                        v119 = (v112 + v106);
                                        v120 = v115;
                                        v121 = (*((int64_t *)(v119)));
                                        v122 = v121;
                                        v123 = (v118 + v106);
                                        v124 = (v120 == 32LL);
                                        if (v124) {
                                            v184 = (*((int32_t *)(v123)));
                                            v185 = v184;
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
                            v264 = (v73 + 12LL);
L4:;
                            /* phi v83 <- (bb5: v73) (bb65: v264) */
                            /* phi v84 <- (bb5: v57) (bb65: v256) */
                            /* phi v85 <- (bb5: v58) (bb65: v251) */
                            v86 = (*((int32_t *)(v83)));
                            v87 = v86;
                            v88 = (v87 & v87);
                            v89 = (v88 != 0LL);
                            if (v89) {
                                goto L2;
                            } else {
                                v90 = (v83 + 4LL);
                                v91 = (*((int32_t *)(v90)));
                                v92 = v91;
                                v93 = (v92 & v92);
                                v94 = (v93 != 0LL);
                                if (v94) {
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
/* struct_layouts: pointer=0 stack=1 */
/* switch_tables: 0 */
int64_t __mingw_raise_matherr(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;
    int8_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = arg3;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = arg2;
    int64_t v14 = arg0;
    int64_t v15 = arg1;
    int64_t v16 = arg4;
    int64_t v17 = arg5;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;

    v1 = (v0 - 88LL);
    v3 = (v2 + 5368742160LL);
    v4 = (*((int64_t *)(v3)));
    v5 = v4;
    v6 = (v5 & v5);
    v7 = (v6 == 0LL);
    if (v7) {
    } else {
        (/* opaque: movsd */ 0);
        (/* opaque: unpcklpd */ 0);
        v8 = (v1 + 32LL);
        *((int32_t *)(v8)) = v9;
        v11 = v8;
        v12 = (v1 + 40LL);
        *((int64_t *)(v12)) = v13;
        (/* opaque: movaps */ 0);
        (/* opaque: movsd */ 0);
        v18 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v14, v15, v13, v11, v16, v17);
    }
    /* phi v19 <- (bb0: v5) (bb2: v18) */
    v20 = (v1 + 88LL);
    return v19;
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
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = arg0;

    v1 = (v0 + 5368742160LL);
    *((int64_t *)(v1)) = v2;
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
int64_t __mingw_SEH_error_handler(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = arg3;
    int64_t v3 = 0LL;
    int8_t v4 = 0LL;
    int64_t v5 = 0LL;
    int8_t v6 = 0LL;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int8_t v10 = 0LL;
    int32_t v11 = 0LL;
    int64_t v12 = 0LL;
    int8_t v13 = 0LL;
    int8_t v14 = 0LL;
    int64_t v15 = 0LL;
    int8_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int32_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    int8_t v25 = 0LL;
    int8_t v26 = 0LL;
    int8_t v27 = 0LL;
    int64_t v28 = arg2;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = arg0;
    int64_t v32 = arg1;
    int64_t v33 = arg4;
    int64_t v34 = arg5;
    int64_t v35 = 0LL;
    int8_t v36 = 0LL;
    int64_t v37 = 0LL;
    int8_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    int64_t v47 = 0LL;
    int8_t v48 = 0LL;
    int64_t v49 = 0LL;
    int8_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int64_t v53 = 0LL;
    int64_t v54 = 0LL;
    int64_t v55 = 0LL;
    int64_t v56 = 0LL;
    int64_t v57 = 0LL;
    int64_t v58 = 0LL;
    int64_t v59 = 0LL;
    int64_t v60 = 0LL;

    v1 = (v0 - 40LL);
    v3 = (v2 + 4LL);
    v4 = (*((int8_t *)(v3)));
    v5 = (v4 & 2LL);
    v6 = (v5 != 0LL);
    if (v6) {
L1:;
        v60 = 1LL;
L0:;
        /* phi v58 <- (bb16: v57) (bb34: v60) (bb36: v44) */
        v59 = (v1 + 40LL);
        return v58;
    } else {
        v7 = 4848615423LL;
        v8 = (*((int64_t *)(v2)));
        v9 = (v7 & v8);
        v10 = (v9 == 541541187LL);
        if (v10) {
L2:;
            /* phi v56 <- (bb1: v9) (bb4: v15) (bb9: v12) (bb15: v40) (bb28: v52) (bb38: v55) (bb41: v43) */
            v57 = (v56 ^ v56);
            goto L0;
        } else {
            v11 = (*((int32_t *)(v2)));
            v12 = v11;
            v13 = (v12 > -1073741674LL);
            if (v13) {
                goto L1;
            } else {
                v14 = (v12 <= -1073741685LL);
                if (v14) {
                    v25 = (v12 == -1073741819LL);
                    if (v25) {
                        v45 = (v28 ^ v28);
                        v46 = 11LL;
                        v47 = ((long long (*)(long long, long long, long long, long long, long long, long long))signal)(v31, v32, v45, v46, v33, v34);
                        v48 = (v47 == 1LL);
                        if (v48) {
                            v53 = 1LL;
                            v54 = 11LL;
                            v55 = ((long long (*)(long long, long long, long long, long long, long long, long long))signal)(v31, v32, v53, v54, v33, v34);
                            goto L2;
                        } else {
                            v49 = (v47 & v47);
                            v50 = (v49 == 0LL);
                            if (v50) {
                                goto L1;
                            } else {
                                v51 = 11LL;
                                v52 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v31, v32, v45, v51, v33, v34);
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
                        v18 = (v17 + 5368726368LL);
                        v19 = v18;
                        v20 = (v15 * 4LL);
                        v21 = (v19 + v20);
                        v22 = (*((int32_t *)(v21)));
                        v23 = v22;
                        v24 = (v23 + v19);
                        /* recovered switch table at block 5 (arm resolution pending) */
                        switch (v15) {
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
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = arg0;
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;
    int32_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int8_t v12 = 0LL;
    int8_t v13 = 0LL;
    int8_t v14 = 0LL;
    int64_t v15 = 0LL;
    int8_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int32_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    int8_t v25 = 0LL;
    int8_t v26 = 0LL;
    int8_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    int8_t v35 = 0LL;
    int64_t v36 = 0LL;
    int8_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int64_t v42 = 0LL;
    int8_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    int8_t v47 = 0LL;
    int64_t v48 = 0LL;
    int8_t v49 = 0LL;
    int64_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int64_t v53 = 0LL;
    int64_t v54 = 0LL;
    int64_t v55 = 0LL;
    int64_t v56 = 0LL;
    int64_t v57 = 0LL;
    int64_t v58 = 0LL;
    int8_t v59 = 0LL;
    int64_t v60 = 0LL;
    int64_t v61 = 0LL;
    int64_t v62 = 0LL;
    int64_t v63 = 0LL;
    int64_t v64 = 0LL;
    int64_t v65 = 0LL;
    int64_t v66 = 0LL;
    int64_t v67 = 0LL;
    int64_t v68 = 0LL;
    int64_t v69 = 0LL;
    int64_t v70 = 0LL;
    int64_t v71 = 0LL;
    int64_t v72 = 0LL;
    int64_t v73 = 0LL;
    int64_t v74 = 0LL;
    int64_t v75 = 0LL;
    int8_t v76 = 0LL;
    int64_t v77 = 0LL;
    int8_t v78 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 32LL);
    v5 = (*((int64_t *)(v4)));
    v6 = v5;
    v7 = (*((int32_t *)(v6)));
    v8 = v7;
    v9 = v4;
    v10 = v8;
    v11 = (v10 & 553648127LL);
    v12 = (v11 == 541541187LL);
    if (v12) {
        v75 = (v6 + 4LL);
        v76 = (*((int8_t *)(v75)));
        v77 = (v76 & 1LL);
        v78 = (v77 != 0LL);
        if (v78) {
L2:;
            v13 = (v8 > -1073741674LL);
            if (v13) {
L0:;
                v55 = (v17 + 5368742192LL);
                v56 = (*((int64_t *)(v55)));
                v57 = v56;
                v58 = (v57 & v57);
                v59 = (v58 == 0LL);
                if (v59) {
                    v65 = (v57 ^ v57);
                    v66 = (v3 + 32LL);
                    v67 = (*((int64_t *)(v66)));
                    v68 = (v66 + 8LL);
                    v69 = v67;
                    return v65;
                } else {
                    v60 = v9;
                    v61 = (v3 + 32LL);
                    v62 = (*((int64_t *)(v61)));
                    v63 = (v61 + 8LL);
                    v64 = v62;
                    /* structurally unreachable: block 9 */
                    __builtin_unreachable();
                }
            } else {
                v14 = (v8 <= -1073741685LL);
                if (v14) {
                    v25 = (v8 == -1073741819LL);
                    if (v25) {
                        v44 = (v6 ^ v6);
                        v45 = 11LL;
                        v46 = ((long long (*)(long long, long long, long long, long long, long long, long long))signal)(v30, v31, v44, v45, v32, v33);
                        v47 = (v46 == 1LL);
                        if (v47) {
                            v52 = 1LL;
                            v53 = 11LL;
                            v54 = ((long long (*)(long long, long long, long long, long long, long long, long long))signal)(v30, v31, v52, v53, v32, v33);
L1:;
                            v70 = -1LL;
                            v71 = (v3 + 32LL);
                            v72 = (*((int64_t *)(v71)));
                            v73 = (v71 + 8LL);
                            v74 = v72;
                            return v70;
                        } else {
                            v48 = (v46 & v46);
                            v49 = (v48 == 0LL);
                            if (v49) {
                                goto L0;
                            } else {
                                v50 = 11LL;
                                v51 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v30, v31, v44, v50, v32, v33);
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
                        v18 = (v17 + 5368726408LL);
                        v19 = v18;
                        v20 = (v15 * 4LL);
                        v21 = (v19 + v20);
                        v22 = (*((int32_t *)(v21)));
                        v23 = v22;
                        v24 = (v23 + v19);
                        /* recovered switch table at block 4 (arm resolution pending) */
                        switch (v15) {
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
/* struct_layouts: pointer=2 stack=1 */
/* switch_tables: 0 */
void __mingwthr_run_key_dtors_part_0(int64_t arg0, int64_t arg1, int64_t arg2) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = arg0;
    int64_t v5 = 0LL;
    int64_t v6 = arg1;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = arg2;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int8_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int32_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    int64_t v35 = 0LL;
    int8_t v36 = 0LL;
    int64_t v37 = 0LL;
    int8_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
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
    int64_t v59 = 0LL;
    int64_t v60 = 0LL;
    int64_t v61 = 0LL;
    int64_t v62 = 0LL;
    int64_t v63 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 8LL);
    *((int64_t *)(v3)) = v4;
    v5 = (v3 - 8LL);
    *((int64_t *)(v5)) = v6;
    v7 = (v5 - 8LL);
    *((int64_t *)(v7)) = v8;
    v9 = (v7 - 40LL);
    v11 = (v10 + 5368742240LL);
    v12 = v11;
    v16 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v4, v6, v13, v12, v14, v15);
    v17 = (v10 + 5368742208LL);
    v18 = (*((int64_t *)(v17)));
    v19 = v18;
    v20 = (v19 & v19);
    v21 = (v20 == 0LL);
    if (v21) {
    } else {
        v22 = (v10 + 5368746728LL);
        v23 = (*((int64_t *)(v22)));
        v24 = v23;
        v25 = (v10 + 5368746656LL);
        v26 = (*((int64_t *)(v25)));
        v27 = v26;
        while (1) {
            /* phi v28 <- (bb2: v6) (bb8: v33) */
            /* phi v29 <- (bb2: v19) (bb8: v46) */
            v30 = (*((int32_t *)(v29)));
            v31 = v30;
            v32 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v27, v28, v13, v31, v14, v15);
            v33 = v32;
            v34 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v27, v33, v13, v31, v14, v15);
            v35 = (v33 & v33);
            v36 = (v35 == 0LL);
            if (v36) {
            } else {
                v37 = (v34 & v34);
                v38 = (v37 != 0LL);
                if (v38) {
                } else {
                    v39 = (v29 + 8LL);
                    v40 = (*((int64_t *)(v39)));
                    v41 = v40;
                    v42 = v33;
                    v43 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v27, v33, v13, v42, v14, v15);
                }
            }
            v44 = (v29 + 16LL);
            v45 = (*((int64_t *)(v44)));
            v46 = v45;
            v47 = (v46 & v46);
            v48 = (v47 != 0LL);
            if (v48) {
                continue;
            }
        }
    }
    v49 = (v10 + 5368742240LL);
    v50 = v49;
    v51 = (v9 + 40LL);
    v52 = (*((int64_t *)(v51)));
    v53 = (v51 + 8LL);
    v54 = v52;
    v55 = (*((int64_t *)(v53)));
    v56 = (v53 + 8LL);
    v57 = v55;
    v58 = (*((int64_t *)(v56)));
    v59 = (v56 + 8LL);
    v60 = v58;
    v61 = (*((int64_t *)(v59)));
    v62 = (v59 + 8LL);
    v63 = v61;
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
/* struct_layouts: pointer=3 stack=0 */
/* switch_tables: 0 */
int64_t ___w64_mingwthr_add_key_dtor(int64_t arg0, int64_t arg1, int64_t arg2) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int32_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = arg0;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int8_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = arg1;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t n = 0LL;
    int64_t size = 0LL;
    int64_t v21 = arg2;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int8_t v24 = 0LL;
    int64_t v25 = 0LL;
    int32_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    int64_t v47 = 0LL;

    v1 = (v0 - 56LL);
    v3 = (v2 + 5368742216LL);
    v4 = (*((int32_t *)(v3)));
    v5 = v4;
    v7 = v6;
    v8 = (v5 & v5);
    v9 = (v8 != 0LL);
    if (v9) {
        v14 = (v1 + 72LL);
        /* recovered field: base=v1 offset=0x48 field=field_48 */
        *((int64_t *)(v14)) = v15;
        v16 = 1LL;
        v17 = 24LL;
        v18 = (v1 + 64LL);
        /* recovered field: base=v1 offset=0x40 field=field_40 */
        *((int32_t *)(v18)) = v7;
        v22 = ((long long (*)(long long, long long, long long, long long, long long, long long))calloc)(n, size, v17, v16, v21, v7);
        v23 = (v22 & v22);
        v24 = (v23 == 0LL);
        if (v24) {
            v47 = -1LL;
        } else {
            v25 = (v1 + 64LL);
            v26 = (*((int32_t *)(v25)));
            v27 = v26;
            v28 = (v1 + 72LL);
            v29 = (*((int64_t *)(v28)));
            v30 = v29;
            v31 = (v1 + 40LL);
            /* recovered field: base=v1 offset=0x28 field=field_28 */
            *((int64_t *)(v31)) = v22;
            v32 = (v2 + 5368742240LL);
            v33 = v32;
            *((int32_t *)(v22)) = v27;
            v34 = (v22 + 8LL);
            /* recovered field: base=v22 offset=0x8 field=field_8 */
            *((int64_t *)(v34)) = v30;
            v35 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(n, size, v17, v33, v30, v27);
            v36 = (v2 + 5368742208LL);
            v37 = (*((int64_t *)(v36)));
            v38 = v37;
            v39 = (v1 + 40LL);
            v40 = (*((int64_t *)(v39)));
            v41 = v40;
            v42 = (v2 + 5368742240LL);
            v43 = v42;
            v44 = (v41 + 16LL);
            *((int64_t *)(v44)) = v38;
            /* recovered field: base=v2 offset=0x140008140 field=field_140008140 */
            *((int64_t *)(v36)) = v41;
            v46 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(n, size, v38, v43, v30, v27);
L0:;
            /* phi v10 <- (bb0: v5) (bb8: v46) */
            v11 = (v10 ^ v10);
        }
    } else {
        goto L0;
    }
    /* phi v12 <- (bb1: v11) (bb9: v47) */
    v13 = (v1 + 56LL);
    return v12;
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
/* struct_layouts: pointer=2 stack=0 */
/* switch_tables: 0 */
int64_t ___w64_mingwthr_remove_key_dtor(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int32_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;
    int8_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = arg0;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t p = 0LL;
    int64_t v15 = 0LL;
    int64_t v16 = arg1;
    int64_t v17 = arg2;
    int64_t v18 = arg3;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int8_t v24 = 0LL;
    int64_t v25 = 0LL;
    int32_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int32_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int8_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int8_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    int64_t v47 = 0LL;
    int64_t v48 = 0LL;
    int64_t v49 = 0LL;
    int64_t v50 = 0LL;
    int64_t v51 = 0LL;

    v1 = (v0 - 40LL);
    v3 = (v2 + 5368742216LL);
    v4 = (*((int32_t *)(v3)));
    v5 = v4;
    v6 = (v5 & v5);
    v7 = (v6 != 0LL);
    if (v7) {
        v10 = (v1 + 48LL);
        *((int32_t *)(v10)) = v11;
        v12 = (v2 + 5368742240LL);
        v13 = v12;
        v19 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(p, v15, v16, v13, v17, v18);
        v20 = (v2 + 5368742208LL);
        v21 = (*((int64_t *)(v20)));
        v22 = v21;
        v23 = (v22 & v22);
        v24 = (v23 == 0LL);
        if (v24) {
        } else {
            v25 = (v1 + 48LL);
            v26 = (*((int32_t *)(v25)));
            v27 = v26;
            v28 = (v17 ^ v17);
            while (1) {
                /* phi v29 <- (bb5: v22) (bb8: v39) */
                /* phi v30 <- (bb5: v28) (bb8: v36) */
                v31 = (*((int32_t *)(v29)));
                v32 = v31;
                v33 = (v29 + 16LL);
                v34 = (*((int64_t *)(v33)));
                v35 = v34;
                /* structurally unreachable: block 9 */
                __builtin_unreachable();
            }
        }
        /* phi v45 <- (bb4: v16) (bb7: v27) (bb12: v27) */
        /* phi v46 <- (bb4: v17) (bb7: v36) (bb12: v30) */
        v47 = (v2 + 5368742240LL);
        v48 = v47;
        v49 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(p, v15, v45, v48, v46, v18);
        v50 = (v49 ^ v49);
        v51 = (v1 + 40LL);
        return v50;
    } else {
        v8 = (v5 ^ v5);
        v9 = (v1 + 40LL);
        return v8;
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
/* struct_layouts: pointer=1 stack=0 */
/* switch_tables: 0 */
int64_t __mingw_TLScallback(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = arg2;
    int8_t v3 = 0LL;
    int64_t v4 = 0LL;
    int8_t v5 = 0LL;
    int64_t v6 = 0LL;
    int64_t v7 = 0LL;
    int32_t v8 = 0LL;
    int64_t v9 = 0LL;
    int64_t v10 = 0LL;
    int8_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = arg0;
    int64_t v16 = arg1;
    int64_t v17 = arg4;
    int64_t v18 = arg5;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int32_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int8_t v24 = 0LL;
    int64_t v25 = 0LL;
    int32_t v26 = 0LL;
    int64_t v27 = 0LL;
    int8_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
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
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int8_t v45 = 0LL;
    int64_t v46 = 0LL;
    int64_t v47 = 0LL;
    int64_t v48 = 0LL;
    int64_t v49 = 0LL;
    int64_t v50 = 0LL;
    int64_t v51 = arg3;
    int64_t v52 = 0LL;
    int64_t v53 = 0LL;
    int64_t v54 = 0LL;
    int8_t v55 = 0LL;
    int64_t v56 = 0LL;
    int32_t v57 = 0LL;
    int64_t v58 = 0LL;
    int64_t v59 = 0LL;
    int8_t v60 = 0LL;
    int64_t v61 = 0LL;
    int64_t v62 = 0LL;
    int64_t v63 = 0LL;
    int64_t v64 = 0LL;

    v1 = (v0 - 56LL);
    v3 = (v2 == 2LL);
    if (v3) {
        v62 = ((long long (*)(long long, long long, long long, long long, long long, long long))_fpreset)(v15, v16, v2, v51, v17, v18);
        v63 = 1LL;
        v64 = (v1 + 56LL);
        return v63;
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
int64_t _ValidateImageBase(int64_t arg0) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
    int64_t v2 = arg0;
    int16_t v3 = 0LL;
    int8_t v4 = 0LL;
    int64_t v5 = 0LL;
    int32_t v6 = 0LL;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int32_t v9 = 0LL;
    int8_t v10 = 0LL;
    int64_t v11 = 0LL;
    int64_t v12 = 0LL;
    int16_t v13 = 0LL;

    v1 = (v0 ^ v0);
    v3 = (*((int16_t *)(v2)));
    v4 = (v3 != 23117LL);
    if (v4) {
        return v1;
    } else {
        v5 = (v2 + 60LL);
        v6 = (*((int32_t *)(v5)));
        v7 = v6;
        v8 = (v2 + v7);
        v9 = (*((int32_t *)(v8)));
        v10 = (v9 == 17744LL);
        if (v10) {
            v11 = (v1 ^ v1);
            v12 = (v8 + 24LL);
            v13 = (*((int16_t *)(v12)));
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
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int16_t v6 = 0LL;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int8_t v9 = 0LL;
    int64_t v10 = 0LL;
    int16_t v11 = 0LL;
    int64_t v12 = 0LL;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int32_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = arg1;
    int8_t v30 = 0LL;
    int64_t v31 = 0LL;
    int32_t v32 = 0LL;
    int64_t v33 = 0LL;
    int8_t v34 = 0LL;
    int64_t v35 = 0LL;
    int8_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;

    v1 = (v0 + 60LL);
    v2 = (*((int32_t *)(v1)));
    v3 = v2;
    v4 = (v3 + v0);
    v5 = (v4 + 6LL);
    v6 = (*((int16_t *)(v5)));
    v7 = v6;
    v8 = (v7 & v7);
    v9 = (v8 == 0LL);
    if (v9) {
L0:;
        /* phi v37 <- (bb0: v4) (bb4: v35) */
        v38 = (v37 ^ v37);
    } else {
        v10 = (v4 + 20LL);
        v11 = (*((int16_t *)(v10)));
        v12 = v11;
        v13 = (v7 - 1LL);
        v14 = (v13 * 4LL);
        v15 = (v13 + v14);
        v16 = v15;
        v17 = (v4 + 24LL);
        v18 = (v17 + v12);
        v19 = v18;
        v20 = (v19 + 40LL);
        v21 = (v16 * 8LL);
        v22 = (v20 + v21);
        v23 = v22;
        while (1) {
            /* phi v24 <- (bb1: v19) (bb4: v35) */
            v25 = (v24 + 12LL);
            v26 = (*((int32_t *)(v25)));
            v27 = v26;
            v28 = v27;
            v30 = (v29 < v27);
            if (v30) {
L1:;
                v35 = (v24 + 40LL);
                v36 = (v35 != v23);
                if (v36) {
                    continue;
                } else {
                    goto L0;
                }
            } else {
                v31 = (v24 + 8LL);
                v32 = (*((int32_t *)(v31)));
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
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = arg0;
    int64_t v5 = 0LL;
    int64_t v6 = arg1;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int64_t v10 = arg3;
    int64_t s = 0LL;
    int64_t v12 = arg2;
    int64_t v13 = arg4;
    int64_t v14 = arg5;
    int64_t v15 = 0LL;
    int8_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int16_t v21 = 0LL;
    int8_t v22 = 0LL;
    int64_t v23 = 0LL;
    int32_t v24 = 0LL;
    int64_t v25 = 0LL;
    int64_t v26 = 0LL;
    int32_t v27 = 0LL;
    int8_t v28 = 0LL;
    int64_t v29 = 0LL;
    int16_t v30 = 0LL;
    int8_t v31 = 0LL;
    int64_t v32 = 0LL;
    int16_t v33 = 0LL;
    int8_t v34 = 0LL;
    int64_t v35 = 0LL;
    int16_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int64_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    int64_t v47 = 0LL;
    int64_t v48 = 0LL;
    int8_t v49 = 0LL;
    int64_t v50 = 0LL;
    int16_t v51 = 0LL;
    int64_t v52 = 0LL;
    int64_t v53 = 0LL;
    int64_t v54 = 0LL;
    int8_t v55 = 0LL;
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
    int64_t v66 = 0LL;
    int64_t v67 = 0LL;
    int64_t v68 = 0LL;
    int64_t v69 = 0LL;
    int64_t v70 = 0LL;
    int64_t v71 = 0LL;
    int64_t v72 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 8LL);
    *((int64_t *)(v3)) = v4;
    v5 = (v3 - 8LL);
    *((int64_t *)(v5)) = v6;
    v7 = (v5 - 8LL);
    *((int64_t *)(v7)) = v8;
    v9 = (v7 - 40LL);
    s = v10;
    v15 = ((long long (*)(long long, long long, long long, long long, long long, long long))strlen)(s, v6, v12, v10, v13, v14);
    v16 = (v15 > 8LL);
    if (v16) {
L0:;
        /* phi v56 <- (bb1: v8) (bb2: v8) (bb6: v8) (bb7: v8) (bb8: v8) (bb13: v54) */
        v57 = (v56 ^ v56);
    } else {
        v18 = (v17 + 5368726528LL);
        v19 = (*((int64_t *)(v18)));
        v20 = v19;
        v21 = (*((int16_t *)(v20)));
        v22 = (v21 == 23117LL);
        if (v22) {
            v23 = (v20 + 60LL);
            v24 = (*((int32_t *)(v23)));
            v25 = v24;
            v26 = (v25 + v20);
            v27 = (*((int32_t *)(v26)));
            v28 = (v27 != 17744LL);
            if (v28) {
                goto L0;
            } else {
                v29 = (v26 + 24LL);
                v30 = (*((int16_t *)(v29)));
                v31 = (v30 != 523LL);
                if (v31) {
                    goto L0;
                } else {
                    v32 = (v26 + 6LL);
                    v33 = (*((int16_t *)(v32)));
                    v34 = (v33 == 0LL);
                    if (v34) {
                        goto L0;
                    } else {
                        v35 = (v26 + 20LL);
                        v36 = (*((int16_t *)(v35)));
                        v37 = v36;
                        v38 = (v6 ^ v6);
                        v39 = (v26 + 24LL);
                        v40 = (v39 + v37);
                        v41 = v40;
                        while (1) {
                            /* phi v42 <- (bb9: v38) (bb12: v53) */
                            /* phi v43 <- (bb9: v41) (bb12: v54) */
                            v44 = 8LL;
                            v45 = s;
                            v46 = v43;
                            v47 = ((long long (*)(long long, long long, long long, long long, long long, long long))strncmp)(s, v42, v45, v46, v44, v14);
                            v48 = (v47 & v47);
                            v49 = (v48 == 0LL);
                            if (v49) {
                            } else {
                                v50 = (v26 + 6LL);
                                v51 = (*((int16_t *)(v50)));
                                v52 = v51;
                                v53 = (v42 + 1LL);
                                v54 = (v43 + 40LL);
                                v55 = (v53 < v52);
                                if (v55) {
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
    /* phi v58 <- (bb3: v57) (bb11: v43) */
    v59 = v58;
    v60 = (v9 + 40LL);
    v61 = (*((int64_t *)(v60)));
    v62 = (v60 + 8LL);
    v63 = v61;
    v64 = (*((int64_t *)(v62)));
    v65 = (v62 + 8LL);
    v66 = v64;
    v67 = (*((int64_t *)(v65)));
    v68 = (v65 + 8LL);
    v69 = v67;
    v70 = (*((int64_t *)(v68)));
    v71 = (v68 + 8LL);
    v72 = v70;
    return v59;
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
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int16_t v6 = 0LL;
    int8_t v7 = 0LL;
    int64_t v8 = 0LL;
    int32_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int32_t v12 = 0LL;
    int8_t v13 = 0LL;
    int64_t v14 = 0LL;
    int16_t v15 = 0LL;
    int8_t v16 = 0LL;
    int64_t v17 = 0LL;
    int16_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int8_t v21 = 0LL;
    int64_t v22 = 0LL;
    int16_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = arg0;
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int32_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int8_t v44 = 0LL;
    int64_t v45 = 0LL;
    int32_t v46 = 0LL;
    int64_t v47 = 0LL;
    int8_t v48 = 0LL;
    int64_t v49 = 0LL;
    int8_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;

    v1 = (v0 + 5368726528LL);
    v2 = (*((int64_t *)(v1)));
    v3 = v2;
    v5 = (v4 ^ v4);
    v6 = (*((int16_t *)(v3)));
    v7 = (v6 != 23117LL);
    if (v7) {
L0:;
        /* phi v52 <- (bb0: v5) (bb1: v5) (bb4: v5) (bb5: v5) (bb8: v39) */
        return v52;
    } else {
        v8 = (v3 + 60LL);
        v9 = (*((int32_t *)(v8)));
        v10 = v9;
        v11 = (v10 + v3);
        v12 = (*((int32_t *)(v11)));
        v13 = (v12 == 17744LL);
        if (v13) {
            v14 = (v11 + 24LL);
            v15 = (*((int16_t *)(v14)));
            v16 = (v15 != 523LL);
            if (v16) {
                goto L0;
            } else {
                v17 = (v11 + 6LL);
                v18 = (*((int16_t *)(v17)));
                v19 = v18;
                v20 = (v19 & v19);
                v21 = (v20 == 0LL);
                if (v21) {
                    goto L0;
                } else {
                    v22 = (v11 + 20LL);
                    v23 = (*((int16_t *)(v22)));
                    v24 = v23;
                    v26 = (v25 - v3);
                    v27 = (v19 + -1LL);
                    v28 = v27;
                    v29 = (v28 * 4LL);
                    v30 = (v28 + v29);
                    v31 = v30;
                    v32 = (v11 + 24LL);
                    v33 = (v32 + v24);
                    v34 = v33;
                    v35 = (v34 + 40LL);
                    v36 = (v31 * 8LL);
                    v37 = (v35 + v36);
                    v38 = v37;
                    while (1) {
                        /* phi v39 <- (bb6: v34) (bb9: v49) */
                        v40 = (v39 + 12LL);
                        v41 = (*((int32_t *)(v40)));
                        v42 = v41;
                        v43 = v42;
                        v44 = (v26 < v42);
                        if (v44) {
L1:;
                            v49 = (v39 + 40LL);
                            v50 = (v49 != v38);
                            if (v50) {
                                continue;
                            } else {
                                v51 = (v49 ^ v49);
                                return v51;
                            }
                        } else {
                            v45 = (v39 + 8LL);
                            v46 = (*((int32_t *)(v45)));
                            v47 = (v43 + v46);
                            v48 = (v26 < v47);
                            if (v48) {
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
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = arg0;
    int64_t v5 = 0LL;
    int16_t v6 = 0LL;
    int8_t v7 = 0LL;
    int64_t v8 = 0LL;
    int32_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int32_t v12 = 0LL;
    int8_t v13 = 0LL;
    int64_t v14 = 0LL;
    int16_t v15 = 0LL;
    int8_t v16 = 0LL;
    int64_t v17 = 0LL;
    int16_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;

    v1 = (v0 + 5368726528LL);
    v2 = (*((int64_t *)(v1)));
    v3 = v2;
    v5 = (v4 ^ v4);
    v6 = (*((int16_t *)(v3)));
    v7 = (v6 != 23117LL);
    if (v7) {
L0:;
        v21 = v5;
        return v21;
    } else {
        v8 = (v3 + 60LL);
        v9 = (*((int32_t *)(v8)));
        v10 = v9;
        v11 = (v3 + v10);
        v12 = (*((int32_t *)(v11)));
        v13 = (v12 == 17744LL);
        if (v13) {
            v14 = (v11 + 24LL);
            v15 = (*((int16_t *)(v14)));
            v16 = (v15 != 523LL);
            if (v16) {
                goto L0;
            } else {
                v17 = (v11 + 6LL);
                v18 = (*((int16_t *)(v17)));
                v19 = v18;
                v20 = v19;
                return v20;
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
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int16_t v6 = 0LL;
    int8_t v7 = 0LL;
    int64_t v8 = 0LL;
    int32_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int32_t v12 = 0LL;
    int8_t v13 = 0LL;
    int64_t v14 = 0LL;
    int16_t v15 = 0LL;
    int8_t v16 = 0LL;
    int64_t v17 = 0LL;
    int16_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int8_t v21 = 0LL;
    int64_t v22 = 0LL;
    int16_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = arg0;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int8_t v41 = 0LL;
    int64_t v42 = 0LL;
    int8_t v43 = 0LL;
    int64_t v44 = 0LL;
    int8_t v45 = 0LL;
    int64_t v46 = 0LL;
    int64_t v47 = 0LL;
    int64_t v48 = 0LL;
    int8_t v49 = 0LL;
    int64_t v50 = 0LL;
    int64_t v51 = 0LL;

    v1 = (v0 + 5368726528LL);
    v2 = (*((int64_t *)(v1)));
    v3 = v2;
    v5 = (v4 ^ v4);
    v6 = (*((int16_t *)(v3)));
    v7 = (v6 != 23117LL);
    if (v7) {
L0:;
        /* phi v51 <- (bb0: v5) (bb1: v5) (bb4: v5) (bb5: v5) (bb8: v38) */
        return v51;
    } else {
        v8 = (v3 + 60LL);
        v9 = (*((int32_t *)(v8)));
        v10 = v9;
        v11 = (v10 + v3);
        v12 = (*((int32_t *)(v11)));
        v13 = (v12 == 17744LL);
        if (v13) {
            v14 = (v11 + 24LL);
            v15 = (*((int16_t *)(v14)));
            v16 = (v15 != 523LL);
            if (v16) {
                goto L0;
            } else {
                v17 = (v11 + 6LL);
                v18 = (*((int16_t *)(v17)));
                v19 = v18;
                v20 = (v19 & v19);
                v21 = (v20 == 0LL);
                if (v21) {
                    goto L0;
                } else {
                    v22 = (v11 + 20LL);
                    v23 = (*((int16_t *)(v22)));
                    v24 = v23;
                    v25 = (v11 + 24LL);
                    v26 = (v25 + v24);
                    v27 = v26;
                    v28 = (v19 + -1LL);
                    v29 = v28;
                    v30 = (v29 * 4LL);
                    v31 = (v29 + v30);
                    v32 = v31;
                    v33 = (v27 + 40LL);
                    v34 = (v32 * 8LL);
                    v35 = (v33 + v34);
                    v36 = v35;
                    while (1) {
                        /* phi v38 <- (bb6: v27) (bb10: v48) */
                        /* phi v39 <- (bb6: v37) (bb10: v47) */
                        v40 = (v38 + 39LL);
                        v41 = (*((int8_t *)(v40)));
                        v42 = (v41 & 32LL);
                        v43 = (v42 == 0LL);
                        if (v43) {
L1:;
                            /* phi v47 <- (bb7: v39) (bb9: v46) */
                            v48 = (v38 + 40LL);
                            v49 = (v36 != v48);
                            if (v49) {
                                continue;
                            } else {
                                v50 = (v48 ^ v48);
                                return v50;
                            }
                        } else {
                            v44 = (v39 & v39);
                            v45 = (v44 == 0LL);
                            if (v45) {
                                break;
                            } else {
                                v46 = (v39 - 1LL);
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
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int16_t v6 = 0LL;
    int8_t v7 = 0LL;
    int64_t v8 = 0LL;
    int32_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int32_t v12 = 0LL;
    int8_t v13 = 0LL;
    int64_t v14 = 0LL;
    int16_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;

    v1 = (v0 + 5368726528LL);
    v2 = (*((int64_t *)(v1)));
    v3 = v2;
    v5 = (v4 ^ v4);
    v6 = (*((int16_t *)(v3)));
    v7 = (v6 != 23117LL);
    if (v7) {
L0:;
        v17 = v5;
        return v17;
    } else {
        v8 = (v3 + 60LL);
        v9 = (*((int32_t *)(v8)));
        v10 = v9;
        v11 = (v10 + v3);
        v12 = (*((int32_t *)(v11)));
        v13 = (v12 == 17744LL);
        if (v13) {
            v14 = (v11 + 24LL);
            v15 = (*((int16_t *)(v14)));
            (/* opaque: cmove */ 0);
            v16 = v5;
            return v16;
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
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int16_t v6 = 0LL;
    int8_t v7 = 0LL;
    int64_t v8 = 0LL;
    int32_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int32_t v12 = 0LL;
    int8_t v13 = 0LL;
    int64_t v14 = 0LL;
    int16_t v15 = 0LL;
    int8_t v16 = 0LL;
    int64_t v17 = 0LL;
    int16_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int8_t v21 = 0LL;
    int64_t v22 = 0LL;
    int16_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = arg0;
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int32_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int8_t v44 = 0LL;
    int64_t v45 = 0LL;
    int32_t v46 = 0LL;
    int64_t v47 = 0LL;
    int8_t v48 = 0LL;
    int64_t v49 = 0LL;
    int32_t v50 = 0LL;
    int64_t v51 = 0LL;
    int64_t v52 = 0LL;
    int64_t v53 = 0LL;
    int64_t v54 = 0LL;
    int8_t v55 = 0LL;
    int64_t v56 = 0LL;

    v1 = (v0 + 5368726528LL);
    v2 = (*((int64_t *)(v1)));
    v3 = v2;
    v5 = (v4 ^ v4);
    v6 = (*((int16_t *)(v3)));
    v7 = (v6 != 23117LL);
    if (v7) {
        return v5;
    } else {
        v8 = (v3 + 60LL);
        v9 = (*((int32_t *)(v8)));
        v10 = v9;
        v11 = (v10 + v3);
        v12 = (*((int32_t *)(v11)));
        v13 = (v12 == 17744LL);
        if (v13) {
            v14 = (v11 + 24LL);
            v15 = (*((int16_t *)(v14)));
            v16 = (v15 != 523LL);
            if (v16) {
L0:;
                goto L0;
            } else {
                v17 = (v11 + 6LL);
                v18 = (*((int16_t *)(v17)));
                v19 = v18;
                v20 = (v19 & v19);
                v21 = (v20 == 0LL);
                if (v21) {
                    goto L0;
                } else {
                    v22 = (v11 + 20LL);
                    v23 = (*((int16_t *)(v22)));
                    v24 = v23;
                    v26 = (v25 - v3);
                    v27 = (v19 + -1LL);
                    v28 = v27;
                    v29 = (v28 * 4LL);
                    v30 = (v28 + v29);
                    v31 = v30;
                    v32 = (v11 + 24LL);
                    v33 = (v32 + v24);
                    v34 = v33;
                    v35 = (v34 + 40LL);
                    v36 = (v31 * 8LL);
                    v37 = (v35 + v36);
                    v38 = v37;
                    while (1) {
                        /* phi v39 <- (bb6: v34) (bb9: v54) */
                        v40 = (v39 + 12LL);
                        v41 = (*((int32_t *)(v40)));
                        v42 = v41;
                        v43 = v42;
                        v44 = (v26 < v42);
                        if (v44) {
L1:;
                            v54 = (v39 + 40LL);
                            v55 = (v38 != v54);
                            if (v55) {
                                continue;
                            } else {
                                break;
                            }
                        } else {
                            v45 = (v39 + 8LL);
                            v46 = (*((int32_t *)(v45)));
                            v47 = (v43 + v46);
                            v48 = (v26 < v47);
                            if (v48) {
                                v49 = (v39 + 36LL);
                                v50 = (*((int32_t *)(v49)));
                                v51 = v50;
                                v52 = ~(v51);
                                v53 = (v52 >> 31LL);
                                return v53;
                            } else {
                                goto L1;
                            }
                        }
                    }
                    v56 = (v54 ^ v54);
                    return v56;
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
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int16_t v6 = 0LL;
    int8_t v7 = 0LL;
    int64_t v8 = 0LL;
    int32_t v9 = 0LL;
    int64_t v10 = 0LL;
    int64_t v11 = 0LL;
    int32_t v12 = 0LL;
    int8_t v13 = 0LL;
    int64_t v14 = 0LL;
    int16_t v15 = 0LL;
    int8_t v16 = 0LL;
    int64_t v17 = 0LL;
    int32_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int8_t v21 = 0LL;
    int64_t v22 = 0LL;
    int16_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int8_t v26 = 0LL;
    int64_t v27 = 0LL;
    int16_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
    int64_t v40 = 0LL;
    int64_t v41 = 0LL;
    int64_t v42 = 0LL;
    int64_t v43 = 0LL;
    int32_t v44 = 0LL;
    int64_t v45 = 0LL;
    int64_t v46 = 0LL;
    int8_t v47 = 0LL;
    int64_t v48 = 0LL;
    int32_t v49 = 0LL;
    int64_t v50 = 0LL;
    int8_t v51 = 0LL;
    int64_t v52 = 0LL;
    int64_t v53 = arg0;
    int64_t v54 = 0LL;
    int64_t v55 = 0LL;
    int64_t v56 = 0LL;
    int32_t v57 = 0LL;
    int64_t v58 = 0LL;
    int64_t v59 = 0LL;
    int8_t v60 = 0LL;
    int64_t v61 = 0LL;
    int32_t v62 = 0LL;
    int64_t v63 = 0LL;
    int64_t v64 = 0LL;
    int8_t v65 = 0LL;
    int64_t v66 = 0LL;
    int8_t v67 = 0LL;
    int64_t v68 = 0LL;
    int64_t v69 = 0LL;
    int64_t v70 = 0LL;
    int32_t v71 = 0LL;
    int64_t v72 = 0LL;
    int64_t v73 = 0LL;
    int64_t v74 = 0LL;
    int64_t v75 = 0LL;
    int64_t v76 = 0LL;
    int8_t v77 = 0LL;
    int64_t v78 = 0LL;
    int64_t v79 = 0LL;
    int64_t v80 = 0LL;
    int64_t v81 = 0LL;

    v1 = (v0 + 5368726528LL);
    v2 = (*((int64_t *)(v1)));
    v3 = v2;
    v5 = (v4 ^ v4);
    v6 = (*((int16_t *)(v3)));
    v7 = (v6 != 23117LL);
    if (v7) {
L0:;
        v81 = v5;
        return v81;
    } else {
        v8 = (v3 + 60LL);
        v9 = (*((int32_t *)(v8)));
        v10 = v9;
        v11 = (v10 + v3);
        v12 = (*((int32_t *)(v11)));
        v13 = (v12 == 17744LL);
        if (v13) {
            v14 = (v11 + 24LL);
            v15 = (*((int16_t *)(v14)));
            v16 = (v15 != 523LL);
            if (v16) {
                goto L0;
            } else {
                v17 = (v11 + 144LL);
                v18 = (*((int32_t *)(v17)));
                v19 = v18;
                v20 = (v19 & v19);
                v21 = (v20 == 0LL);
                if (v21) {
                    goto L0;
                } else {
                    v22 = (v11 + 6LL);
                    v23 = (*((int16_t *)(v22)));
                    v24 = v23;
                    v25 = (v24 & v24);
                    v26 = (v25 == 0LL);
                    if (v26) {
                        goto L0;
                    } else {
                        v27 = (v11 + 20LL);
                        v28 = (*((int16_t *)(v27)));
                        v29 = v28;
                        v30 = (v11 + 24LL);
                        v31 = (v30 + v29);
                        v32 = v31;
                        v33 = (v24 + -1LL);
                        v34 = v33;
                        v35 = (v34 * 4LL);
                        v36 = (v34 + v35);
                        v37 = v36;
                        v38 = (v32 + 40LL);
                        v39 = (v37 * 8LL);
                        v40 = (v38 + v39);
                        v41 = v40;
                        while (1) {
                            /* phi v42 <- (bb7: v32) (bb10: v76) */
                            v43 = (v42 + 12LL);
                            v44 = (*((int32_t *)(v43)));
                            v45 = v44;
                            v46 = v45;
                            v47 = (v19 < v45);
                            if (v47) {
L2:;
                                /* phi v75 <- (bb8: v46) (bb9: v50) */
                                v76 = (v42 + 40LL);
                                v77 = (v41 != v76);
                                if (v77) {
                                    continue;
                                } else {
                                    break;
                                }
                            } else {
                                v48 = (v42 + 8LL);
                                v49 = (*((int32_t *)(v48)));
                                v50 = (v46 + v49);
                                v51 = (v19 < v50);
                                if (v51) {
                                    v52 = (v19 + v3);
                                    while (1) {
                                        /* phi v54 <- (bb13: v52) (bb15: v69) */
                                        /* phi v55 <- (bb13: v53) (bb15: v68) */
                                        v56 = (v54 + 4LL);
                                        v57 = (*((int32_t *)(v56)));
                                        v58 = v57;
                                        v59 = (v58 & v58);
                                        v60 = (v59 != 0LL);
                                        if (v60) {
L1:;
                                            v66 = (v55 & v55);
                                            v67 = (v66 > 0LL);
                                            if (v67) {
                                                v68 = (v55 - 1LL);
                                                v69 = (v54 + 20LL);
                                                continue;
                                            } else {
                                                v70 = (v54 + 12LL);
                                                v71 = (*((int32_t *)(v70)));
                                                v72 = v71;
                                                v73 = (v72 + v3);
                                                v74 = v73;
                                                return v74;
                                            }
                                        } else {
                                            v61 = (v54 + 12LL);
                                            v62 = (*((int32_t *)(v61)));
                                            v63 = v62;
                                            v64 = (v63 & v63);
                                            v65 = (v64 == 0LL);
                                            if (v65) {
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
                        /* phi v78 <- (bb10: v75) (bb17: v58) */
                        v79 = (v78 ^ v78);
                        v80 = v79;
                        return v80;
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
    int64_t v1 = 0LL;
    int64_t v2 = arg0;
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
    int8_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 8LL);
    *((int64_t *)(v3)) = v4;
    v5 = (v3 + 24LL);
    v6 = v5;
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
    int64_t v1 = 0LL;
    int64_t v2 = arg0;
    int64_t v3 = 0LL;
    int64_t v4 = arg1;
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;
    int64_t v7 = 0LL;
    int64_t v8 = arg3;
    int64_t v9 = 0LL;
    int64_t v10 = arg2;
    int64_t v11 = 0LL;
    int64_t v12 = arg4;
    int64_t v13 = 0LL;
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
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 8LL);
    *((int64_t *)(v3)) = v4;
    v5 = (v3 - 8LL);
    *((int64_t *)(v5)) = v6;
    v7 = (v5 - 48LL);
    v9 = v8;
    v11 = v10;
    v13 = v12;
    v15 = ((long long (*)(long long, long long, long long, long long, long long, long long))__local_stdio_printf_options)(v13, v11, v10, v8, v12, v14);
    v16 = (v14 ^ v14);
    v17 = v11;
    v18 = v9;
    v19 = (*((int64_t *)(v15)));
    v20 = v19;
    v21 = (v7 + 32LL);
    /* recovered field: base=v7 offset=0x20 field=field_20 */
    *((int64_t *)(v21)) = v13;
    v22 = ((long long (*)(long long, long long, long long, long long, long long, long long))__stdio_common_vfprintf)(v13, v11, v18, v20, v17, v16);
    v23 = (v7 + 48LL);
    v24 = (*((int64_t *)(v23)));
    v25 = (v23 + 8LL);
    v26 = v24;
    v27 = (*((int64_t *)(v25)));
    v28 = (v25 + 8LL);
    v29 = v27;
    v30 = (*((int64_t *)(v28)));
    v31 = (v28 + 8LL);
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
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;
    int64_t v6 = 0LL;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
    int64_t v10 = arg0;
    int64_t v11 = 0LL;
    int64_t v12 = arg1;
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = arg2;
    int64_t v16 = 0LL;
    int64_t v17 = arg3;
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
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int64_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 8LL);
    *((int64_t *)(v3)) = v4;
    v5 = (v3 - 8LL);
    *((int64_t *)(v5)) = v6;
    v7 = (v5 - 64LL);
    v8 = (v7 + 112LL);
    v9 = v8;
    v11 = v10;
    v13 = v12;
    /* recovered field: base=v7 offset=0x70 field=field_70 */
    *((int64_t *)(v8)) = v15;
    v16 = (v7 + 120LL);
    *((int64_t *)(v16)) = v17;
    v18 = (v7 + 56LL);
    *((int64_t *)(v18)) = v9;
    v19 = ((long long (*)(long long, long long, long long, long long, long long, long long))__local_stdio_printf_options)(v9, v13, v12, v10, v15, v17);
    v20 = (v17 ^ v17);
    v21 = v13;
    v22 = v11;
    v23 = (*((int64_t *)(v19)));
    v24 = v23;
    v25 = (v7 + 32LL);
    /* recovered field: base=v7 offset=0x20 field=field_20 */
    *((int64_t *)(v25)) = v9;
    v26 = ((long long (*)(long long, long long, long long, long long, long long, long long))__stdio_common_vfprintf)(v9, v13, v22, v24, v21, v20);
    v27 = (v7 + 64LL);
    v28 = (*((int64_t *)(v27)));
    v29 = (v27 + 8LL);
    v30 = v28;
    v31 = (*((int64_t *)(v29)));
    v32 = (v29 + 8LL);
    v33 = v31;
    v34 = (*((int64_t *)(v32)));
    v35 = (v32 + 8LL);
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
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;

    v1 = (v0 + 5368721504LL);
    v2 = v1;
    return v2;
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
    int64_t v4 = 0LL;
    int64_t v5 = 0LL;

    v1 = (v0 + 5368726608LL);
    v2 = (*((int64_t *)(v1)));
    v3 = v2;
    v4 = (*((int64_t *)(v3)));
    v5 = v4;
    return v5;
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
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
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
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 32LL);
    v5 = v4;
    v6 = 2LL;
    v12 = ((long long (*)(long long, long long, long long, long long, long long, long long))__acrt_iob_func)(v7, v8, v9, v6, v10, v11);
    v13 = v5;
    v15 = (v14 + 5368726448LL);
    v16 = v15;
    v17 = v12;
    v18 = ((long long (*)(long long, long long, long long, long long, long long, long long))fprintf)(v7, v8, v16, v17, v13, v11);
    v19 = 255LL;
    v20 = ((long long (*)(long long, long long, long long, long long, long long, long long))_exit)(v7, v8, v16, v19, v13, v11);
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
    int64_t v1 = 0LL;
    int64_t v2 = 0LL;
    int64_t v3 = 0LL;
    int64_t v4 = arg0;
    int64_t v5 = 0LL;
    int64_t v6 = arg1;
    int64_t v7 = 0LL;
    int64_t v8 = 0LL;
    int64_t v9 = 0LL;
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
    int64_t v22 = 0LL;
    int32_t v23 = 0LL;
    int64_t v24 = 0LL;
    int64_t v25 = 0LL;
    int64_t v26 = 0LL;
    int64_t v27 = 0LL;
    int64_t v28 = 0LL;
    int64_t v29 = 0LL;
    int64_t v30 = 0LL;
    int64_t v31 = 0LL;
    int64_t v32 = 0LL;
    int64_t v33 = 0LL;
    int32_t v34 = 0LL;
    int64_t v35 = 0LL;
    int64_t v36 = 0LL;
    int64_t v37 = 0LL;
    int64_t v38 = 0LL;
    int64_t v39 = 0LL;
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
    int64_t v50 = 0LL;

    v1 = (v0 - 8LL);
    *((int64_t *)(v1)) = v2;
    v3 = (v1 - 8LL);
    *((int64_t *)(v3)) = v4;
    v5 = (v3 - 8LL);
    *((int64_t *)(v5)) = v6;
    v7 = (v5 - 8LL);
    *((int64_t *)(v7)) = v8;
    v9 = (v7 - 40LL);
    v11 = v10;
    v13 = v12;
    v15 = v14;
    v17 = v16;
    v18 = ((long long (*)(long long, long long, long long, long long, long long, long long))_initialize_narrow_environment)(v17, v13, v12, v16, v14, v10);
    v19 = 1LL;
    v20 = (v19 - -1LL);
    v21 = ((long long (*)(long long, long long, long long, long long, long long, long long))_configure_narrow_argv)(v17, v13, v12, v20, v14, v10);
    v22 = ((long long (*)(long long, long long, long long, long long, long long, long long))__p___argc)(v17, v13, v12, v20, v14, v10);
    v23 = (*((int32_t *)(v22)));
    v24 = v23;
    *((int32_t *)(v17)) = v24;
    v25 = ((long long (*)(long long, long long, long long, long long, long long, long long))__p___argv)(v17, v13, v12, v20, v14, v10);
    v26 = (*((int64_t *)(v25)));
    v27 = v26;
    *((int64_t *)(v13)) = v27;
    v28 = ((long long (*)(long long, long long, long long, long long, long long, long long))__p__environ)(v17, v13, v12, v20, v14, v10);
    v29 = (*((int64_t *)(v28)));
    v30 = v29;
    *((int64_t *)(v15)) = v30;
    v31 = (v9 + 112LL);
    v32 = (*((int64_t *)(v31)));
    v33 = v32;
    v34 = (*((int32_t *)(v33)));
    v35 = v34;
    v36 = ((long long (*)(long long, long long, long long, long long, long long, long long))_set_new_mode)(v17, v13, v12, v35, v14, v10);
    v37 = (v36 ^ v36);
    v38 = (v9 + 40LL);
    v39 = (*((int64_t *)(v38)));
    v40 = (v38 + 8LL);
    v41 = v39;
    v42 = (*((int64_t *)(v40)));
    v43 = (v40 + 8LL);
    v44 = v42;
    v45 = (*((int64_t *)(v43)));
    v46 = (v43 + 8LL);
    v47 = v45;
    v48 = (*((int64_t *)(v46)));
    v49 = (v46 + 8LL);
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
/* struct_layouts: pointer=0 stack=0 */
/* switch_tables: 0 */
int64_t main(int64_t arg0, int64_t arg1, int64_t arg2, int64_t arg3, int64_t arg4, int64_t arg5) {
    int64_t v0 = 0LL;
    int64_t v1 = 0LL;
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
    int64_t v13 = 0LL;
    int64_t v14 = 0LL;
    int64_t v15 = 0LL;
    int64_t v16 = 0LL;
    int64_t v17 = 0LL;
    int64_t v18 = 0LL;
    int64_t v19 = 0LL;
    int64_t v20 = 0LL;
    int64_t v21 = 0LL;
    int64_t v22 = 0LL;
    int64_t v23 = 0LL;
    int64_t v24 = 0LL;

    v1 = (v0 - 88LL);
    v8 = ((long long (*)(long long, long long, long long, long long, long long, long long))__main)(v2, v3, v4, v5, v6, v7);
    v9 = -11LL;
    v10 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v2, v3, v4, v9, v6, v7);
    (/* opaque: movaps */ 0);
    v11 = (v4 ^ v4);
    v12 = (v9 ^ v9);
    v13 = (v1 + 56LL);
    *((int32_t *)(v13)) = v11;
    v15 = v13;
    v16 = (v1 + 61LL);
    v17 = v16;
    v18 = 18LL;
    v19 = (v1 + 32LL);
    *((int64_t *)(v19)) = v12;
    v20 = v10;
    (/* opaque: movups */ 0);
    v21 = (v1 + 76LL);
    *((int32_t *)(v21)) = 673104LL;
    v22 = ((long long (*)(long long, long long, long long, long long, long long, long long))(/* opaque: indirect-call */ 0))(v2, v3, v17, v20, v18, v15);
    v23 = 42LL;
    v24 = (v1 + 88LL);
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
