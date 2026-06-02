/* dac --target c -O1 reconstruction
   input: tests/fixtures/hello-x86_64
   arch:  x86-64 */
#include <stdint.h>
#include <stddef.h>

/* dac-recovered function stub */
/* address: 0x1000 */
/* end: 0x101b */
/* confidence: 1.00 (Observed) */
void _init(void) {
    /* lifter→SSA bridge pending; body intentionally empty */
}

/* dac-recovered function stub */
/* address: 0x1030 */
/* end: 0x1040 */
/* confidence: 0.85 (Derived) */
void fn_1030(void) {
    /* lifter→SSA bridge pending; body intentionally empty */
}

/* dac-recovered function stub */
/* address: 0x1040 */
/* end: 0x105e */
/* confidence: 1.00 (Observed) */
void main(void) {
    /* lifter→SSA bridge pending; body intentionally empty */
}

/* dac-recovered function stub */
/* address: 0x1060 */
/* end: 0x1086 */
/* confidence: 1.00 (Observed) */
void _start(void) {
    /* lifter→SSA bridge pending; body intentionally empty */
}

/* dac-recovered function stub */
/* address: 0x1090 */
/* end: 0x10c0 */
/* confidence: 1.00 (Observed) */
void deregister_tm_clones(void) {
    /* lifter→SSA bridge pending; body intentionally empty */
}

/* dac-recovered function stub */
/* address: 0x10c0 */
/* end: 0x1100 */
/* confidence: 1.00 (Observed) */
void register_tm_clones(void) {
    /* lifter→SSA bridge pending; body intentionally empty */
}

/* dac-recovered function stub */
/* address: 0x1100 */
/* end: 0x1150 */
/* confidence: 1.00 (Observed) */
void __do_global_dtors_aux(void) {
    /* lifter→SSA bridge pending; body intentionally empty */
}

/* dac-recovered function stub */
/* address: 0x1150 */
/* end: 0x1159 */
/* confidence: 1.00 (Observed) */
void frame_dummy(void) {
    /* lifter→SSA bridge pending; body intentionally empty */
}

/* dac-recovered function stub */
/* address: 0x115c */
/* end: 0x1169 */
/* confidence: 1.00 (Observed) */
void _fini(void) {
    /* lifter→SSA bridge pending; body intentionally empty */
}
